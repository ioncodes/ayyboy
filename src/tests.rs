#[cfg(test)]
mod tests {
    use crate::lr35902::cpu::*;
    use crate::lr35902::sm83::*;
    use crate::memory::mmu::*;
    use serde_json::Value;

    fn is_ignore(_path: &std::path::Path) -> bool {
        false
    }

    #[datatest::files("./external/sm83/v1", {
        input in r"^.*\.json" if !is_ignore
    })]
    fn test_cpu(input: &str) {
        let tests: Value = serde_json::from_str(&input).unwrap();

        for test in tests.as_array().unwrap() {
            let mut mmu = Mmu::new(vec![], vec![0u8; 0xffff]);
            mmu.unmap_bootrom();
            mmu.resize_memory(0xffff * 4);
            let mut sm83 = Sm83::new();
            let mut cpu = Cpu::new();

            let test = test.as_object().unwrap();
            let name = test.get("name").unwrap().as_str().unwrap();
            let initial = test.get("initial").unwrap().as_object().unwrap();
            let final_state = test.get("final").unwrap().as_object().unwrap();

            cpu.write_register(&Register::A, initial.get("a").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::F, initial.get("f").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::B, initial.get("b").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::C, initial.get("c").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::D, initial.get("d").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::E, initial.get("e").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::H, initial.get("h").unwrap().as_u64().unwrap() as u8);
            cpu.write_register(&Register::L, initial.get("l").unwrap().as_u64().unwrap() as u8);
            cpu.write_register16(&Register::SP, initial.get("sp").unwrap().as_u64().unwrap() as u16);
            cpu.write_register16(&Register::PC, initial.get("pc").unwrap().as_u64().unwrap() as u16);

            let ram = initial.get("ram").unwrap().as_array().unwrap();
            for value in ram {
                let addr = value.as_array().unwrap()[0].as_u64().unwrap() as u16;
                let value = value.as_array().unwrap()[1].as_u64().unwrap() as u8;

                mmu.write(addr, value);
            }

            if let Ok(instruction) = sm83.decode(&mut mmu, cpu.read_register16(&Register::PC)) {
                println!("{}", instruction);
            } else {
                panic!("Failed to decode instruction");
            };

            cpu.tick(&mut mmu);

            assert_eq!(
                cpu.read_register(&Register::A),
                final_state.get("a").unwrap().as_u64().unwrap() as u8,
                "Comparison with register A failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::F),
                final_state.get("f").unwrap().as_u64().unwrap() as u8,
                "Comparison with register F failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::B),
                final_state.get("b").unwrap().as_u64().unwrap() as u8,
                "Comparison with register B failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::C),
                final_state.get("c").unwrap().as_u64().unwrap() as u8,
                "Comparison with register C failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::D),
                final_state.get("d").unwrap().as_u64().unwrap() as u8,
                "Comparison with register D failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::E),
                final_state.get("e").unwrap().as_u64().unwrap() as u8,
                "Comparison with register E failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::H),
                final_state.get("h").unwrap().as_u64().unwrap() as u8,
                "Comparison with register H failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register(&Register::L),
                final_state.get("l").unwrap().as_u64().unwrap() as u8,
                "Comparison with register L failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register16(&Register::SP),
                final_state.get("sp").unwrap().as_u64().unwrap() as u16,
                "Comparison with register SP failed for {}",
                name
            );
            assert_eq!(
                cpu.read_register16(&Register::PC),
                final_state.get("pc").unwrap().as_u64().unwrap() as u16,
                "Comparison with register PC failed for {}",
                name
            );

            let ram = final_state.get("ram").unwrap().as_array().unwrap();
            for value in ram {
                let addr = value.as_array().unwrap()[0].as_u64().unwrap() as u16;
                let value = value.as_array().unwrap()[1].as_u64().unwrap() as u8;

                assert_eq!(mmu.read(addr), value, "Comparison with RAM failed for {}", name);
            }
        }
    }
}
