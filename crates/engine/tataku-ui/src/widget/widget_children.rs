use crate::widget::*;

#[derive(Default)]
pub enum WidgetChildren<'a, Action> {
    #[default] None,
    #[allow(clippy::borrowed_box, reason = "signature")]
    Single(&'a Box<dyn Widget<Action>>),
    List(&'a [Box<dyn Widget<Action>>]),
    #[allow(clippy::borrowed_box, reason = "signature")]
    OwnedList(Vec<&'a Box<dyn Widget<Action>>>),
}
impl<'a, Action> IntoIterator for WidgetChildren<'a, Action> {
    type Item = &'a Box<dyn Widget<Action>>;
    type IntoIter = WidgetChildrenIter<'a, Action>;
    fn into_iter(self) -> Self::IntoIter {
        WidgetChildrenIter(self, 0)
    }
}


pub struct WidgetChildrenIter<'a, Action>(WidgetChildren<'a, Action>, usize);
impl<'a, Action> Iterator for WidgetChildrenIter<'a, Action> {
    type Item = &'a Box<dyn Widget<Action>>;
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
pub enum WidgetChildrenMut<'a, Action> {
    #[default] None,
    Single(&'a mut Box<dyn Widget<Action>>),
    List(&'a mut [Box<dyn Widget<Action>>]),
    OwnedList(Vec<&'a mut Box<dyn Widget<Action>>>),
}
impl<'a, Action> IntoIterator for WidgetChildrenMut<'a, Action> {
    type Item = &'a mut Box<dyn Widget<Action>>;
    type IntoIter = WidgetChildrenIterMut<'a, Action>;
    fn into_iter(self) -> Self::IntoIter {
        WidgetChildrenIterMut(self, 0)
    }
}

pub struct WidgetChildrenIterMut<'a, Action>(WidgetChildrenMut<'a, Action>, usize);
impl<'a, Action> Iterator for WidgetChildrenIterMut<'a, Action> {
    type Item = &'a mut Box<dyn Widget<Action>>;
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
                        &mut Box<dyn Widget<Action>>, 
                        &mut Box<dyn Widget<Action>>
                    >(a) 
                })
            }

            WidgetChildrenMut::OwnedList(list) => {
                let a = list.get_mut(self.1)?;
                self.1 += 1;

                // SAFETY: can never return two mutable references at the same time
                Some(unsafe { 
                    std::mem::transmute::<
                        &mut Box<dyn Widget<Action>>, 
                        &mut Box<dyn Widget<Action>>
                    >(a) 
                })
            } 
        }
    }
}
