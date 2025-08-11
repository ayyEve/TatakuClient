use crate::prelude::*;

#[derive(Default)]
pub enum WidgetChildren<'a> {
    #[default] None,
    #[allow(clippy::borrowed_box, reason = "signature")]
    Single(&'a Box<dyn Widget>),
    List(&'a [Box<dyn Widget>]),
    #[allow(clippy::borrowed_box, reason = "signature")]
    OwnedList(Vec<&'a Box<dyn Widget>>),
}
impl<'a> IntoIterator for WidgetChildren<'a> {
    type Item = &'a Box<dyn Widget>;
    type IntoIter = WidgetChildrenIter<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WidgetChildrenIter(self, 0)
    }
}


pub struct WidgetChildrenIter<'a>(WidgetChildren<'a>, usize);
impl<'a> Iterator for WidgetChildrenIter<'a> {
    type Item = &'a Box<dyn Widget>;
    fn next(&mut self) -> Option<Self::Item> {
        if let WidgetChildren::OwnedList(list) = &self.0 {
            let a = list.get(self.1);
            self.1 += 1;
            return a.map(|i| &**i);
        }

        match &self.0 {
            WidgetChildren::None => None,
            WidgetChildren::Single(_) => {
                let WidgetChildren::Single(i) = std::mem::take(&mut self.0)
                else { unreachable!() };
                Some(i)
            }
            WidgetChildren::List(list) => {
                let a = list.get(self.1);
                self.1 += 1;
                a
            }
            WidgetChildren::OwnedList(_) => unreachable!(),
        }
    }
}



#[derive(Default)]
pub enum WidgetChildrenMut<'a> {
    #[default] None,
    Single(&'a mut Box<dyn Widget>),
    List(&'a mut [Box<dyn Widget>]),
    OwnedList(Vec<&'a mut Box<dyn Widget>>),
}
impl<'a> IntoIterator for WidgetChildrenMut<'a> {
    type Item = &'a mut Box<dyn Widget>;
    type IntoIter = WidgetChildrenIterMut<'a>;
    fn into_iter(self) -> Self::IntoIter {
        WidgetChildrenIterMut(self, 0)
    }
}

pub struct WidgetChildrenIterMut<'a>(WidgetChildrenMut<'a>, usize);
impl<'a> Iterator for WidgetChildrenIterMut<'a> {
    type Item = &'a mut Box<dyn Widget>;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            WidgetChildrenMut::None => None,
            WidgetChildrenMut::Single(i) => {
                if self.1 == 0 {
                    self.1 += 1;
                    
                    // SAFETY: can never return two mutable references at the same time
                    Some(unsafe { std::ptr::read(i) })
                } else {
                    None
                }
            }
            WidgetChildrenMut::List(list) => {
                let a = list.get_mut(self.1)?;
                self.1 += 1;

                // SAFETY: can never return two mutable references at the same time
                Some(unsafe { 
                    std::mem::transmute::<
                        &mut Box<dyn Widget>, 
                        &mut Box<dyn Widget>
                    >(a) 
                })
            }

            WidgetChildrenMut::OwnedList(list) => {
                let a = list.get_mut(self.1)?;
                self.1 += 1;

                // SAFETY: can never return two mutable references at the same time
                Some(unsafe { 
                    std::mem::transmute::<
                        &mut Box<dyn Widget>, 
                        &mut Box<dyn Widget>
                    >(a) 
                })
            } 
        }
    }
}
