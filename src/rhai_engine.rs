use crate::lr35902::cpu::Cpu;
use crate::lr35902::sm83::Register;
use crate::memory::mmu::Mmu;
use rhai::{Engine, Scope, AST};
use std::path::PathBuf;

pub struct RhaiEngine<'a> {
    rhai: Engine,
    rhai_scope: Scope<'a>,
    rhai_script: AST,
}

impl<'a> RhaiEngine<'a> {
    pub fn new(path: PathBuf) -> RhaiEngine<'a> {
        let mut rhai = Engine::new();

        rhai.register_fn("set_register", |cpu: &mut Cpu, register: i32, value: i64| match register {
            0 => cpu.write_register16(&Register::AF, value as u16),
            1 => cpu.write_register16(&Register::BC, value as u16),
            2 => cpu.write_register16(&Register::DE, value as u16),
            3 => cpu.write_register16(&Register::HL, value as u16),
            4 => cpu.write_register16(&Register::SP, value as u16),
            5 => cpu.write_register16(&Register::PC, value as u16),
            6 => cpu.write_register(&Register::A, value as u8),
            7 => cpu.write_register(&Register::F, value as u8),
            8 => cpu.write_register(&Register::B, value as u8),
            9 => cpu.write_register(&Register::C, value as u8),
            10 => cpu.write_register(&Register::D, value as u8),
            11 => cpu.write_register(&Register::E, value as u8),
            12 => cpu.write_register(&Register::H, value as u8),
            13 => cpu.write_register(&Register::L, value as u8),
            _ => panic!("Invalid register: {}", register),
        });
        rhai.register_fn("get_register", |cpu: &mut Cpu, register: i32| match register {
            0 => cpu.read_register16(&Register::AF) as i64,
            1 => cpu.read_register16(&Register::BC) as i64,
            2 => cpu.read_register16(&Register::DE) as i64,
            3 => cpu.read_register16(&Register::HL) as i64,
            4 => cpu.read_register16(&Register::SP) as i64,
            5 => cpu.read_register16(&Register::PC) as i64,
            6 => cpu.read_register(&Register::A) as i64,
            7 => cpu.read_register(&Register::F) as i64,
            8 => cpu.read_register(&Register::B) as i64,
            9 => cpu.read_register(&Register::C) as i64,
            10 => cpu.read_register(&Register::D) as i64,
            11 => cpu.read_register(&Register::E) as i64,
            12 => cpu.read_register(&Register::H) as i64,
            13 => cpu.read_register(&Register::L) as i64,
            _ => panic!("Invalid register: {}", register),
        });
        rhai.register_fn("read_memory", |mmu: &mut Mmu, addr: i32| mmu.read_unchecked(addr as u16) as i64);
        rhai.register_fn("write_memory", |mmu: &mut Mmu, addr: i32, data: i64| {
            mmu.write_unchecked(addr as u16, data as u8)
        });

        let rhai_scope = Scope::new();
        let result = rhai.compile_file(path);
        if let Err(e) = result {
            panic!("Error: {}", e);
        }
        let rhai_script = result.unwrap();

        RhaiEngine {
            rhai,
            rhai_scope,
            rhai_script,
        }
    }

    pub fn prepare_scope(&mut self, cpu: &Cpu, mmu: &Mmu) {
        self.rhai_scope.clear();
        self.rhai_scope.push("cpu", cpu.clone());
        self.rhai_scope.push("mmu", mmu.clone());
        self.rhai_scope.push("REG_AF", 0);
        self.rhai_scope.push("REG_BC", 1);
        self.rhai_scope.push("REG_DE", 2);
        self.rhai_scope.push("REG_HL", 3);
        self.rhai_scope.push("REG_SP", 4);
        self.rhai_scope.push("REG_PC", 5);
        self.rhai_scope.push("REG_A", 6);
        self.rhai_scope.push("REG_F", 7);
        self.rhai_scope.push("REG_B", 8);
        self.rhai_scope.push("REG_C", 9);
        self.rhai_scope.push("REG_D", 10);
        self.rhai_scope.push("REG_E", 11);
        self.rhai_scope.push("REG_H", 12);
        self.rhai_scope.push("REG_L", 13);
    }

    pub fn get_hw_from_scope(&self) -> (Cpu, Mmu) {
        (self.rhai_scope.get_value("cpu").unwrap(), self.rhai_scope.get_value("mmu").unwrap())
    }

    pub fn execute_script(&mut self) {
        let result = self.rhai.eval_ast_with_scope::<()>(&mut self.rhai_scope, &self.rhai_script);
        if let Err(e) = result {
            panic!("Error: {}", e);
        }
    }
}
