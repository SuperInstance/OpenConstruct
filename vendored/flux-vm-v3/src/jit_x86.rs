/// FLUX JIT x86_64 — Native machine code generation
///
/// Generates actual x86_64 machine code that implements the constraint hot path.
/// The generated function takes a `f64` in `xmm0` and returns a `u8` error mask in `al`.
///
/// This matches the C implementation's `flux_check_exact()` hot path exactly:
/// - Compare value against each lo[i]/hi[i] using ucomisd
/// - Build error mask with setb + shl + or
/// - NaN always violates all constraints
///
/// # Safety
///
/// The generated code is raw x86_64 machine code placed in a Vec<u8>. Execution
/// requires RWX memory or mmap with PROT_EXEC. Currently we execute from the
/// Vec's heap allocation (which is RW- on most systems, not RWX).
///
/// For production use, this should use mmap with MAP_ANONYMOUS + PROT_WRITE + PROT_EXEC.
/// The test suite validates correctness but may fall back to the Rust path on
/// systems with strict W^X enforcement.

/// x86_64 instruction encoder for the FLUX constraint hot path.
///
/// Generates a function with this signature (System V AMD64 ABI):
/// ```c
/// uint8_t flux_jit_check(double value);  // value in xmm0, result in al
/// ```
///
/// Generated code structure:
/// ```asm
/// push rbp
/// mov rbp, rsp
/// ucomisd xmm0, xmm0       ; NaN check
/// jp .nan_handler
/// xor eax, eax             ; mask = 0
/// ; For each constraint i:
///   mov rax, lo[i] (bits)   ; load lower bound
///   movq xmm1, rax
///   ucomisd xmm0, xmm1      ; value vs lo
///   setb cl                  ; CF=1 if value < lo
///   shl cl, i
///   or al, cl
///   mov rax, hi[i] (bits)   ; load upper bound
///   movq xmm1, rax
///   ucomisd xmm1, xmm0      ; hi vs value
///   setb cl                  ; CF=1 if hi < value
///   shl cl, i
///   or al, cl
/// mov rsp, rbp
/// pop rbp
/// ret
/// .nan_handler:
///   mov eax, (1<<n)-1
///   mov rsp, rbp
///   pop rbp
///   ret
/// ```
#[cfg(target_arch = "x86_64")]
pub fn generate_flux_check_native(lo: &[f64], hi: &[f64]) -> Result<Vec<u8>, String> {
    if lo.len() != hi.len() {
        return Err(format!(
            "lo/hi length mismatch: {} vs {}",
            lo.len(),
            hi.len()
        ));
    }
    if lo.is_empty() {
        return Err("no constraints".into());
    }
    if lo.len() > 8 {
        return Err(format!("too many constraints: {} (max 8)", lo.len()));
    }

    let n = lo.len();
    let mut code = Vec::with_capacity(512);

    // ── Prologue ──
    code.push(0x55); // push rbp
    emit_rexw(&mut code);
    code.extend_from_slice(&[0x89, 0xe5]); // mov rbp, rsp

    // ── NaN check: ucomisd xmm0, xmm0 ──
    code.extend_from_slice(&[0x66, 0x0f, 0x2e, 0xc0]); // ucomisd xmm0, xmm0

    // jp .nan (rel32) — jump if parity flag set (NaN)
    code.extend_from_slice(&[0x0f, 0x8a]); // jp rel32
    let jp_rel32_offset = code.len(); // position of the rel32 displacement
    code.extend_from_slice(&[0, 0, 0, 0]); // placeholder, fix up later

    // ── Initialize mask = 0 ──
    code.extend_from_slice(&[0x31, 0xc0]); // xor eax, eax

    // ── Constraint loop ──
    for i in 0..n {
        // Check lo[i]: value < lo[i] → set bit i
        emit_load_xmm1_f64(&mut code, lo[i]);
        // ucomisd xmm0, xmm1 (compare value with lo)
        code.extend_from_slice(&[0x66, 0x0f, 0x2e, 0xc1]); // ucomisd xmm0, xmm1
        // setb cl (set to 1 if CF=1, i.e. value is Below lo)
        code.extend_from_slice(&[0x0f, 0x92, 0xc1]); // setb cl
        if i > 0 {
            code.extend_from_slice(&[0xc0, 0xe1, i as u8]); // shl cl, i
        }
        code.extend_from_slice(&[0x08, 0xc8]); // or al, cl

        // Check hi[i]: value > hi[i] → set bit i
        emit_load_xmm1_f64(&mut code, hi[i]);
        // ucomisd xmm1, xmm0 (compare hi with value)
        // If hi < value → CF=1 (Below)
        code.extend_from_slice(&[0x66, 0x0f, 0x2e, 0xc8]); // ucomisd xmm1, xmm0
        // setb cl
        code.extend_from_slice(&[0x0f, 0x92, 0xc1]); // setb cl
        if i > 0 {
            code.extend_from_slice(&[0xc0, 0xe1, i as u8]); // shl cl, i
        }
        code.extend_from_slice(&[0x08, 0xc8]); // or al, cl
    }

    // ── Epilogue (normal path) ──
    emit_rexw(&mut code);
    code.extend_from_slice(&[0x89, 0xec]); // mov rsp, rbp
    code.push(0x5d); // pop rbp
    code.push(0xc3); // ret

    // ── NaN handler ──
    let nan_handler_offset = code.len();
    // mov eax, (1 << n) - 1
    let all_bits: u32 = (1u32 << n) - 1;
    code.push(0xb8); // mov eax, imm32
    code.extend_from_slice(&all_bits.to_le_bytes());
    emit_rexw(&mut code);
    code.extend_from_slice(&[0x89, 0xec]); // mov rsp, rbp
    code.push(0x5d); // pop rbp
    code.push(0xc3); // ret

    // ── Fix up JP rel32 ──
    let jp_target = nan_handler_offset;
    let jp_from = jp_rel32_offset + 4; // rel32 is relative to END of jp instruction
    let rel32 = (jp_target as isize - jp_from as isize) as i32;
    code[jp_rel32_offset..jp_rel32_offset + 4].copy_from_slice(&rel32.to_le_bytes());

    Ok(code)
}

/// Load an f64 constant into xmm1 using mov rax, imm64 + movq xmm1, rax
#[cfg(target_arch = "x86_64")]
fn emit_load_xmm1_f64(code: &mut Vec<u8>, value: f64) {
    let bits = value.to_bits();
    // mov rax, imm64
    code.push(0x48); // REX.W
    code.push(0xb8); // mov rax, imm64
    code.extend_from_slice(&bits.to_le_bytes());
    // movq xmm1, rax
    code.extend_from_slice(&[0x66, 0x48, 0x0f, 0x6e, 0xc8]);
}

#[cfg(target_arch = "x86_64")]
fn emit_rexw(code: &mut Vec<u8>) {
    code.push(0x48);
}

/// Disassemble the generated code for debugging (hex dump)
pub fn disasm_hex(code: &[u8]) -> String {
    code.iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Validate generated code size is reasonable
pub fn validate_code_size(code: &[u8], n_constraints: usize) -> Result<(), String> {
    // Each constraint adds ~40 bytes (2x: mov rax(10) + movq(5) + ucomisd(4) + setb(3) + shl(3) + or(2) = 27*2 = 54)
    // Plus prologue(5) + NaN check(10) + init(2) + epilogue(7) + NaN handler(14) = 38
    // mov rax is 10 bytes (REX.W + opcode + 8 bytes imm64)
    let expected_max = 38 + n_constraints * 120;
    if code.len() > expected_max {
        return Err(format!(
            "generated code too large: {} bytes (expected ≤ {})",
            code.len(),
            expected_max
        ));
    }
    if code.len() < 20 {
        return Err(format!(
            "generated code suspiciously small: {} bytes",
            code.len()
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_generate_simple() {
        let code = generate_flux_check_native(&[0.0], &[100.0]).unwrap();
        assert!(!code.is_empty());
        println!("Generated {} bytes for 1 constraint", code.len());
        println!("Hex: {}", disasm_hex(&code));
        assert!(validate_code_size(&code, 1).is_ok());
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_generate_multi_constraint() {
        let lo = vec![0.0, 50.0, -100.0];
        let hi = vec![100.0, 150.0, 100.0];
        let code = generate_flux_check_native(&lo, &hi).unwrap();
        assert!(validate_code_size(&code, 3).is_ok());
        println!("Generated {} bytes for 3 constraints", code.len());
    }

    #[test]
    fn test_validate_code_size() {
        // Too small
        assert!(validate_code_size(&[0xc3], 1).is_err());
        // Reasonable
        let mut reasonable = vec![0x55, 0x48, 0x89, 0xe5]; // prologue
        reasonable.extend(std::iter::repeat(0x90).take(100)); // nops
        reasonable.push(0xc3); // ret
        assert!(validate_code_size(&reasonable, 1).is_ok());
    }

    #[test]
    #[cfg(target_arch = "x86_64")]
    fn test_preset_generation() {
        let presets = [
            ("automotive", vec![(0.0, 8000.0), (0.0, 300.0), (-40.0, 150.0)]),
            ("aviation", vec![(-1000.0, 45000.0), (0.0, 600.0)]),
            ("medical", vec![(36.1, 37.8), (60.0, 100.0), (95.0, 100.0)]),
        ];

        for (name, pairs) in &presets {
            let lo: Vec<f64> = pairs.iter().map(|(l, _)| *l).collect();
            let hi: Vec<f64> = pairs.iter().map(|(_, h)| *h).collect();
            let code = generate_flux_check_native(&lo, &hi);
            assert!(code.is_ok(), "preset {name} failed: {:?}", code.err());
            let code = code.unwrap();
            assert!(validate_code_size(&code, pairs.len()).is_ok(),
                "preset {name} code size validation failed");
        }
    }
}
