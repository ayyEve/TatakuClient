use crate::prelude::*;

#[cfg(feature="graphics")]
#[allow(clippy::wrong_self_convention)]
pub trait MakeSettingsMenu {
    fn into_elements(
        &self, 
        prefix: String,
        owner: MessageOwner, 
        builder: &mut SettingsBuilder<'_>,
    );

    fn from_elements(
        &mut self,
        // tags of the current property, with all previous prefixes removed 
        tags: &mut ReflectPath,
        // message that contains the data
        message: Message,
        
        shell: &mut GenericShell,
    );
}
