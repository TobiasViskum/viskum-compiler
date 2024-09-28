use data_structures::Either;

use crate::{
    cfg_visitor::{ walk_basic_block, CfgVisitor },
    Cfg,
    Icfg,
    Operand,
    OperandKind,
    PlaceKind,
};
use std::fmt::{ Display, Write };

const INDENTATION: usize = 4;

pub struct IcfgPrettifier<'b> {
    icfg: &'b Icfg,
    buffer: String,
}

impl<'b> IcfgPrettifier<'b> {
    pub fn new(icfg: &'b Icfg) -> Self {
        Self { icfg, buffer: String::with_capacity(2048) }
    }

    pub fn print_icfg(&mut self) {
        match self.visit_cfg(&self.icfg.cfgs[0]) {
            Ok(_) => {}
            Err(e) => panic!("{}", e),
        }

        println!("{}", self.buffer)
    }

    fn display_place_kind(place: &PlaceKind, cfg: &Cfg) -> String {
        let mut temp_buffer = String::with_capacity(8);
        (|| -> Result<(), std::fmt::Error> {
            match place {
                PlaceKind::LocalMemId(local_mem_id) => {
                    write!(temp_buffer, "{}", cfg.get_local_mem(*local_mem_id))
                }
                PlaceKind::ResultMemId(result_mem_id) => {
                    write!(temp_buffer, "{}", cfg.get_result_mem(*result_mem_id))
                }
                PlaceKind::TempId(temp_id) => { write!(temp_buffer, "{}", temp_id) }
            }
        })().expect("Unexpected write error");
        temp_buffer
    }

    fn dislay_operand(operand: &Operand, cfg: &Cfg) -> String {
        let mut temp_buffer = String::with_capacity(8);

        (|| -> Result<(), std::fmt::Error> {
            write!(temp_buffer, "(")?;

            match &operand.kind {
                OperandKind::Place(place) => {
                    let displayed_place = Self::display_place_kind(&PlaceKind::TempId(*place), cfg);
                    temp_buffer += displayed_place.as_str();
                }
                OperandKind::Const(const_val) => write!(temp_buffer, "{}", const_val)?,
            }

            write!(temp_buffer, " as {})", operand.ty)
        })().expect("Unexpected write error");

        temp_buffer
    }
}

impl<'b> CfgVisitor for IcfgPrettifier<'b> {
    type Result = Result<(), std::fmt::Error>;

    fn default_result() -> Self::Result {
        Ok(())
    }

    fn visit_local_mem(&mut self, local_mem: &crate::LocalMem) -> Self::Result {
        writeln!(self.buffer, "{}declare {}: {}", " ".repeat(INDENTATION), local_mem, local_mem.ty)
    }
    fn visit_result_mem(&mut self, result_mem: &crate::ResultMem) -> Self::Result {
        writeln!(
            self.buffer,
            "{}declare {}: {}",
            " ".repeat(INDENTATION),
            result_mem.result_mem_id,
            result_mem.ty
        )
    }

    fn visit_basic_block(
        &mut self,
        basic_block: &crate::BasicBlock,
        cfg: &crate::Cfg
    ) -> Self::Result {
        write!(self.buffer, "bb{}: {{\n", basic_block.basic_block_id.0)?;
        walk_basic_block(self, basic_block, cfg)?;
        write!(self.buffer, "}}\n")?;

        Self::default_result()
    }

    fn visit_binary_node(
        &mut self,
        binary_node: &crate::BinaryNode,
        cfg: &crate::Cfg
    ) -> Self::Result {
        writeln!(
            self.buffer,
            "{}{}: {} = {} {} {}",
            " ".repeat(INDENTATION),
            binary_node.result_place,
            binary_node.result_ty,
            Self::dislay_operand(&binary_node.lhs, cfg),
            binary_node.op,
            Self::dislay_operand(&binary_node.rhs, cfg)
        )
    }

    fn visit_branch_node(&mut self, branch_node: &crate::BranchNode, _cfg: &Cfg) -> Self::Result {
        writeln!(self.buffer, "{}br bb{}", " ".repeat(INDENTATION), branch_node.branch.0)
    }

    fn visit_branch_cond_node(
        &mut self,
        branch_cond_node: &crate::BranchCondNode,
        _cfg: &Cfg
    ) -> Self::Result {
        writeln!(
            self.buffer,
            "{}br {}, bb{}, bb{}",
            " ".repeat(INDENTATION),
            branch_cond_node.condition,
            branch_cond_node.true_branch.0,
            branch_cond_node.false_branch.0
        )
    }

    fn visit_load_node(&mut self, load_node: &crate::LoadNode, cfg: &Cfg) -> Self::Result {
        let place = &(match load_node.loc_id {
            Either::Left(local_mem_id) => PlaceKind::LocalMemId(local_mem_id),
            Either::Right(result_mem_id) => PlaceKind::ResultMemId(result_mem_id),
        });

        writeln!(
            self.buffer,
            "{}{}: {} = load {}",
            " ".repeat(INDENTATION),
            load_node.result_place,
            load_node.ty,
            Self::display_place_kind(place, cfg)
        )
    }

    fn visit_init_node(&mut self, init_node: &crate::StoreNode, cfg: &crate::Cfg) -> Self::Result {
        let place = &(match init_node.setter {
            Either::Left(local_mem_id) => PlaceKind::LocalMemId(local_mem_id),
            Either::Right(result_mem_id) => PlaceKind::ResultMemId(result_mem_id),
        });

        writeln!(
            self.buffer,
            "{}{}: {} = {}",
            " ".repeat(INDENTATION),
            Self::display_place_kind(place, cfg),
            init_node.result_ty,
            Self::dislay_operand(&init_node.value, cfg)
        )
    }
}