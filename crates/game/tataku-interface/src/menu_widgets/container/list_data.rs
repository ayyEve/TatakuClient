use crate::prelude::*;

#[derive(ChainableInitializer)]
pub struct ProgrammaticListData {
    /// What element to build for each iteration
    #[chain] pub template: Element,

    /// What variable to iterate over
    #[chain] pub list_var: engine::VariablePathResolver,

    /// What var name to store the iter variable in (ie the `i` in `for i in ...`)
    #[chain] pub variable: ArcStr,

    pub(super) error_printed: bool,
}
impl ProgrammaticListData {
    pub fn new(template: Element, list_var: ArcStr, variable: ArcStr) -> Self {
        Self {
            template,
            list_var: engine::VariablePathResolver::new(list_var),
            variable,
            error_printed: false,
        }
    }

    pub(super) fn print_err(&mut self, error: &dyn std::fmt::Debug) {
        if self.error_printed { return }

        self.error_printed = true;
        error!("!!!!!!!!!!!!!!");
        error!("List variable error! '{:?}' {error:?}", self.list_var);
        error!("!!!!!!!!!!!!!!");
    }

}