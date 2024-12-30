
use crate::{ cfg_visitor::{ walk_basic_block, CfgVisitor }, Cfg, Icfg, Operand, PlaceKind };
use std::fmt::Write;

const INDENTATION: usize = 4;

pub struct IcfgPrettifier<'b> {
    icfg: &'b Icfg<'b>,
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
        {
            match place {
                // PlaceKind::GlobalMemId(global_mem_id) => {
                //     write!(temp_buffer, "{}", cfg.get_global_mem(*global_mem_id))
                // }
                PlaceKind::LocalMemId(local_mem_id) => {
                    write!(temp_buffer, "{}", cfg.get_local_mem(*local_mem_id))
                }
                PlaceKind::ResultMemId(result_mem_id) => {
                    write!(temp_buffer, "{}", cfg.get_result_mem(*result_mem_id))
                }
                PlaceKind::TempId(temp_id) => { write!(temp_buffer, "{}", temp_id) }
            }
        }.expect("Unexpected write error");
        temp_buffer
    }

    fn dislay_operand(operand: &Operand, cfg: &Cfg) -> String {
        let mut temp_buffer = String::with_capacity(8);

        (|| -> Result<(), std::fmt::Error> {
            write!(temp_buffer, "(")?;

            match &operand {
                Operand::PlaceKind(place) => {
                    let displayed_place = Self::display_place_kind(place, cfg);
                    write!(temp_buffer, "{}", displayed_place)?;
                }
                Operand::Const(const_val) => write!(temp_buffer, "{}", const_val)?,
            }

            // write!(temp_buffer, " as {})", operand.ty)
            write!(temp_buffer, ")")
        })().expect("Unexpected write error");

        temp_buffer
    }
}

impl CfgVisitor for IcfgPrettifier<'_> {
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
        writeln!(self.buffer, "bb{}: {{", basic_block.basic_block_id.0)?;
        walk_basic_block(self, basic_block, cfg)?;
        writeln!(self.buffer, "}}")?;

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
            binary_node.op_ty,
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
        writeln!(
            self.buffer,
            "{}{}: {} = load {}",
            " ".repeat(INDENTATION),
            load_node.result_place,
            load_node.load_ty,
            Self::display_place_kind(&load_node.load_place, cfg)
        )
    }

    fn visit_store_node(
        &mut self,
        store_node: &crate::StoreNode,
        cfg: &crate::Cfg
    ) -> Self::Result {
        writeln!(
            self.buffer,
            "{}{}: {} = {}",
            " ".repeat(INDENTATION),
            Self::display_place_kind(&store_node.setter, cfg),
            store_node.op_ty,
            Self::dislay_operand(&store_node.value, cfg)
        )
    }
}
