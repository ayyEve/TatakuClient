use crate::widget::*;

#[derive(Default)]
pub enum WidgetChildren<'a, Action> {
    #[default] None,
    Single(&'a dyn Widget<Action>),
    List(&'a [Box<dyn Widget<Action>>]),
    OwnedList(Vec<&'a dyn Widget<Action>>),
}
impl<'a, Action> IntoIterator for WidgetChildren<'a, Action> {
    type Item = &'a dyn Widget<Action>;
    type IntoIter = WidgetChildrenIter<'a, Action>;
    fn into_iter(self) -> Self::IntoIter {
        WidgetChildrenIter(self, 0)
    }
}


pub struct WidgetChildrenIter<'a, Action>(WidgetChildren<'a, Action>, usize);
impl<'a, Action> Iterator for WidgetChildrenIter<'a, Action> {
    type Item = &'a dyn Widget<Action>;
    fn next(&mut self) -> Option<Self::Item> {
        match &self.0 {
            WidgetChildren::None => None,
            WidgetChildren::Single(a) => {
                if self.1 == 0 {
                    self.1 += 1;

                    Some(*a)
                } else {
                    None
                }
            }

            WidgetChildren::List(list) => {
                let a = list.get(self.1);
                self.1 += 1;

                a.map(|a| &**a)
            }
            WidgetChildren::OwnedList(list) => {
                let a = list.get(self.1);
                self.1 += 1;
                a.map(|a| &**a)
            },
        }
    }
}



#[derive(Default)]
pub enum WidgetChildrenMut<'a, Action> {
    #[default] None,
    Single(&'a mut dyn Widget<Action>),
    List(&'a mut [Box<dyn Widget<Action>>]),
    OwnedList(Vec<&'a mut dyn Widget<Action>>),
}
impl<'a, Action> IntoIterator for WidgetChildrenMut<'a, Action> {
    type Item = &'a mut dyn Widget<Action>;
    type IntoIter = WidgetChildrenIterMut<'a, Action>;
    fn into_iter(self) -> Self::IntoIter {
        WidgetChildrenIterMut(self, 0)
    }
}

pub struct WidgetChildrenIterMut<'a, Action>(WidgetChildrenMut<'a, Action>, usize);
impl<'a, Action> Iterator for WidgetChildrenIterMut<'a, Action> {
    type Item = &'a mut dyn Widget<Action>;
    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            WidgetChildrenMut::None => None,
            WidgetChildrenMut::Single(a) => {
                if self.1 == 0 {
                    let a = &mut **a;
                    self.1 += 1;
                    
                    // SAFETY: can never return two mutable references at the same time
                    Some(unsafe { reborrow(a) })
                } else {
                    None
                }
            }
            WidgetChildrenMut::List(list) => {
                let a = &mut **list.get_mut(self.1)?;
                self.1 += 1;

                // SAFETY: can never return two mutable references at the same time
                Some(unsafe { reborrow(a) })
            }

            WidgetChildrenMut::OwnedList(list) => {
                let a = &mut **list.get_mut(self.1)?;
                self.1 += 1;

                // SAFETY: can never return two mutable references at the same time
                Some(unsafe { reborrow(a) })
            } 
        }
    }
}

unsafe fn reborrow<'a, 'b, T: ?Sized>(value: &'a mut T) -> &'b mut T {
    unsafe {
        std::mem::transmute::<
            &'a mut T,
            &'b mut T,
        >(value)
    }
}
