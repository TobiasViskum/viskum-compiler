use crate::{ CallNode, Cfg, CfgVisitor };

pub struct CfgAnalyzer<'a, 'b> where 'a: 'b {
    pub cfg: &'b mut Cfg<'a>,
}

impl<'a, 'b> CfgVisitor for CfgAnalyzer<'a, 'b> where 'a: 'b {
    type Result = ();

    fn default_result() -> Self::Result {}

    fn visit_call_node(&mut self, call_node: &CallNode, cfg: &Cfg) -> Self::Result {}
}
