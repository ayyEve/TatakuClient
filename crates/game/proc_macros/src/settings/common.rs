use syn::{ meta::ParseNestedMeta, * };


const SKIP_ATTRIBUTE:&str = "skip";
const TOOLTIP_ATTRIBUTE:&str = "tooltip";
const TEXT_ATTRIBUTE:&str = "text";
#[derive(Clone, Debug, Default)]
pub(super) struct CommonItems {
    pub add_item: bool,

    /// What text to display
    pub text: String,
    pub skip: bool,
    pub tooltip: Option<String>,
}
impl CommonItems {
    pub fn try_read(&mut self, meta: &ParseNestedMeta) -> Result<bool> {
        if meta.path.is_ident(TEXT_ATTRIBUTE) {
            let _ = meta.value()?;
            let value: LitStr = meta.input.parse()?;
            self.text = value.value();

            Ok(true)
        } else if meta.path.is_ident(TOOLTIP_ATTRIBUTE) {
            let _ = meta.value()?;
            let value: LitStr = meta.input.parse()?;
            self.tooltip = Some(value.value());
            
            Ok(true)
        } else if meta.path.is_ident(SKIP_ATTRIBUTE) {
            let _ = meta.value()?;
            let value: LitBool = meta.input.parse()?;
            self.skip = value.value;

            Ok(true)
        }
        else { Ok(false) }
    }
}
