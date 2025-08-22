use crate::prelude::*;


#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, PartialEq)]
pub enum BuildableUiAction {
    Operate {
        target: BuildableUiOperationTargetTag,
        operation: BuildableUiOperationTypeTag,
    },
}
impl BuildableUiAction {
    pub fn into_action(
        &self, 
        node: NodeId,
        values: &dyn Reflect,
        passed_in: Option<&TatakuValue>,
    ) -> Option<UiActionType> {
        match self {
            Self::Operate {
                operation,
                target
            } => {
                Some(UiOperation {
                    owner: node.owner,
                    target: target.resolve(node, values)?,
                    operation: operation.resolve(values, passed_in)?,
                }.into())
            }
        }
    }

    pub fn build(
        &mut self,
        values: &dyn Reflect,
    ) {
        match self {
            Self::Operate {
                operation,
                target,
            } => {
                if let Err(e) = target.build(values) {
                    error!("error building target: {e:?}");
                }

                operation.build(values);
            },
        }
    }
}


#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableUiOperationTarget {
    /// Self
    #[default] 
    #[serde(alias="self")]
    Node,

    Parent,

    Id { 
        #[serde(rename="@id", default)] attribute: Option<ArcStr>,
        #[serde(rename="$value", default)] tag: Option<BuildableTextTag>,
    },
    Class {
        #[serde(rename="@class", default)] attribute: Option<ArcStr>,
        #[serde(rename="$value", default)] tag: Option<BuildableTextTag>,
    },
}
impl BuildableUiOperationTarget {
    pub fn build(&mut self, _values: &dyn Reflect) -> ShuntingYardResult<()> {
        match self {
            Self::Class { 
                tag: Some(tag), 
                .. 
            } => tag.compute()?,

            Self::Id { 
                tag: Some(tag), 
                .. 
            } => tag.compute()?,

            _ => {}
        }
        Ok(())
    }

    pub fn resolve(
        &self, 
        node: NodeId,
        values: &dyn Reflect,
    ) -> Option<UiOperationTarget> {
        match self {
            Self::Node => Some(UiOperationTarget::Node(node)),
            Self::Parent => Some(UiOperationTarget::Parent(node)),
            
            Self::Id { 
                attribute, 
                tag 
            } => {
                Some(UiOperationTarget::ElementId(
                    text_from_attr_or_tag(
                        attribute.as_ref(), 
                        tag.as_deref(), 
                        values
                    )?.into()
                ))
            }

            Self::Class { 
                attribute, 
                tag 
            } => {
                Some(UiOperationTarget::ElementClass(
                    text_from_attr_or_tag(
                        attribute.as_ref(), 
                        tag.as_deref(), 
                        values
                    )?.into()
                ))
            }
        }
    }
}
crate::impl_tag!(BuildableUiOperationTargetTag, BuildableUiOperationTarget);


#[derive(Deserialize)]
#[serde(rename_all="camelCase")]
#[derive(Clone, Debug, Default, PartialEq)]
pub enum BuildableUiOperationType {
    #[default] None, 
    Scroll {
        #[serde(rename="$value")]
        scroll: BuildableScrollOperation,
    },
    State {
        #[serde(rename="$value")]
        state: BuildableStateOperation,
    }
}
impl BuildableUiOperationType {
    fn resolve(
        &self, 
        values: &dyn Reflect, 
        passed_in: Option<&TatakuValue>
    ) -> Option<UiOperationType> {
        match self {
            Self::None => None,
            Self::Scroll { 
                scroll,
            } => {
                Some(UiOperationType::Scroll(
                    scroll.resolve(values, passed_in)?
                ))
            }
            Self::State { 
                state 
            } => {
                Some(UiOperationType::State(state.resolve()?))
            }
        }
    }
    
    fn build(&mut self, values: &dyn Reflect) {
        match self {
            Self::Scroll { 
                scroll,
            } => {
                scroll.build(values);
            }

            Self::State { .. } => {}
            Self::None => {}
        }
    }
}
crate::impl_tag!(BuildableUiOperationTypeTag, BuildableUiOperationType);

fn text_from_attr_or_tag(
    attr: Option<&ArcStr>,
    tag: Option<&BuildableText>,
    values: &dyn Reflect,
) -> Option<String> {
    match (attr, tag) {
        (Some(_), Some(_)) => panic!("both attribute and tag specified!"),
        (Some(a), _) => Some(a.to_string()),
        (_, Some(a)) => Some(a.to_string(values)),
        _ => None
    }
}


pub use scroll::*;
pub use state::*;
mod scroll {
    use super::*;

    #[derive(Deserialize)]
    #[derive(Clone, Debug, Default, PartialEq)]
    #[serde(rename_all="camelCase")]
    pub struct BuildableScrollOperation {
        #[serde(rename="$value")]
        pub scroll_to: BuildableScrollOperationType,
    }
    impl BuildableScrollOperation {
        pub fn resolve(
            &self, 
            values: &dyn Reflect, 
            passed_in: Option<&TatakuValue>
        ) -> Option<ScrollOperation> {
            let scroll_type = self.scroll_to
                .resolve(values, passed_in)?;

            Some(ScrollOperation {
                scroll_type
            })
        }
        
        pub fn build(&mut self, values: &dyn Reflect) {
            self.scroll_to.build(values);
        }
    }
    crate::impl_tag!(BuildableScrollOperationTag, BuildableScrollOperation);

    #[derive(Deserialize)]
    #[serde(rename_all="camelCase")]
    #[derive(Clone, Debug, Default, PartialEq)]
    pub enum BuildableScrollOperationType {
        #[default] None,

        /// scroll to the (first) active child element
        /// 
        /// include_children indicates if we should check the children's children [...] to find the selected item
        Active {
            #[serde(rename="@include_children", default)]
            include_children: bool,
        },

        /// scroll to the item with this id
        Id {
            #[serde(rename="@id", default)] attr: Option<ArcStr>,
            #[serde(rename="$value", default)] tag: Option<BuildableText>,
        },

        Absolute {
            x: BuildableValueTag,
            y: BuildableValueTag,
        },
        Relative {
            x: BuildableValueTag,
            y: BuildableValueTag,
        },

        AbsolutePercent {
            x: BuildableValueTag,
            y: BuildableValueTag,
        },
        RelativePercent {
            x: BuildableValueTag,
            y: BuildableValueTag,
        },
    }
    impl BuildableScrollOperationType {
        pub fn resolve(
            &self, 
            values: &dyn Reflect, 
            passed_in: Option<&TatakuValue>
        ) -> Option<ScrollType> {
            macro_rules! parse_xy {
                ($x: expr, $y: expr) => {
                    Vector2::new(
                        $x.resolve(values, passed_in)?.as_f32()?,
                        $y.resolve(values, passed_in)?.as_f32()?,
                    )
                }
            }

            match self {
                Self:: None => None,
                Self::Active {
                    include_children,
                } => Some(ScrollType::ScrollToActive {
                    include_children: *include_children,
                }),
                Self::Id { 
                    attr,
                    tag 
                } => {
                    let id = text_from_attr_or_tag(
                        attr.as_ref(), 
                        tag.as_ref(), 
                        values
                    )?;

                    Some(ScrollType::ScrollToId(id.into()))
                }

                Self::Absolute { x, y } => {
                    Some(ScrollType::ScrollToPosition(parse_xy!(x, y)))
                }
                Self::Relative { x, y } => {
                    Some(ScrollType::ScrollByAmount(parse_xy!(x, y)))
                }

                Self::AbsolutePercent { x, y } => {
                    Some(ScrollType::ScrollToPercent(parse_xy!(x, y)))
                }
                Self::RelativePercent { x, y } => {
                    Some(ScrollType::ScrollByPercent(parse_xy!(x, y)))
                }
            }
        }
        
        pub fn build(&mut self, values: &dyn Reflect) {
            match self {
                Self::Id { 
                    tag: Some(tag), 
                    .. 
                } => { 
                    if let Err(e) = tag.compute() {
                        error!("error building id: {e:?}");
                    }
                },

                Self::Absolute { x, y } 
                | Self::Relative { x, y }
                | Self::AbsolutePercent { x, y }
                | Self::RelativePercent { x, y }
                => {
                    x.resolve_pre(values);
                    y.resolve_pre(values);
                }

                _ => {}
            }
        }
    }

    crate::impl_tag!(BuildableScrollOperationTypeTag, BuildableScrollOperationType);
}

mod state {
    use super::*;

    #[derive(Deserialize)]
    #[serde(rename_all="camelCase")]
    #[derive(Clone, Debug, Default, PartialEq)]
    pub enum BuildableStateOperation {
        #[default] None,
        Add {
            states: Vec<BuildableElementState>,
        },
        Remove {
            states: Vec<BuildableElementState>,
        },
        Toggle {
            states: Vec<BuildableElementState>,
        },
    }
    impl BuildableStateOperation {
        fn resolve_states(states: &[BuildableElementState]) -> ElementState {
            states.iter().fold(
                ElementState::empty(),
                |i, n| i | (*n).into()
            )
        }
    
        pub fn resolve(&self) -> Option<StateOperation> {
            match self {
                Self::None => None,
                Self::Add { 
                    states 
                } => Some(StateOperation::Add(Self::resolve_states(states))),
                
                Self::Remove { 
                    states 
                } => Some(StateOperation::Remove(Self::resolve_states(states))),

                Self::Toggle { 
                    states 
                } => Some(StateOperation::Toggle(Self::resolve_states(states))),
            }
        }
    }
    crate::impl_tag!(BuildableStateOperationTag, BuildableStateOperation);
    
    #[derive(Deserialize)]
    #[serde(rename_all="camelCase")]
    #[derive(Copy, Clone, Debug, Default, PartialEq)]
    pub enum BuildableElementState {
        #[default] None,
        Active,
        Hover,
        Focus,
    }
    impl From<BuildableElementState> for ElementState {
        fn from(value: BuildableElementState) -> Self {
            match value {
                BuildableElementState::None => ElementState::None,
                BuildableElementState::Hover => ElementState::Hover,
                BuildableElementState::Focus => ElementState::Focus,
                BuildableElementState::Active => ElementState::Active,
            }
        }
    } 

}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scroll() {
        let scroll = r#"
            <action><ui>
                <operate>
                    <target>
                        <id id="target_id"/>
                    </target>
                    <operation>
                        <scroll>
                            <scrollTo>
                                <active />
                            </scrollTo>
                        </scroll>
                    </operation>
                </operate>
            </ui></action>
        "#;
    
        let a = quick_xml::de::from_str::<BuildableActionTag>(scroll)
            .unwrap();
        assert_eq!(
            a,
            BuildableActionTag::new(BuildableAction::Ui {
                action: BuildableUiAction::Operate { 
                    target: BuildableUiOperationTarget::Id { 
                        attribute: Some("target_id".into()), 
                        tag: None,
                    }.into(),
                    operation: BuildableUiOperationType::Scroll { 
                        scroll: BuildableScrollOperation { 
                            scroll_to: BuildableScrollOperationType::Active {
                                include_children: false
                            }
                        }
                    }.into()
                }
            })
        );
    }


    
    #[test]
    fn test_scroll2() {
        let scroll = r#"
            <action><ui>
                <operate>
                    <target>
                        <id id="group-list" />
                    </target>
                    <operation>
                        <scroll>
                            <scrollTo>
                                <active include_children="true" />
                            </scrollTo>
                        </scroll>
                    </operation>
                </operate>
            </ui></action>
        "#;
    
        let a = quick_xml::de::from_str::<BuildableActionTag>(scroll)
            .unwrap();
        assert_eq!(
            a,
            BuildableActionTag::new(BuildableAction::Ui {
                action: BuildableUiAction::Operate { 
                    target: BuildableUiOperationTarget::Id { 
                        attribute: Some("group-list".into()), 
                        tag: None,
                    }.into(),
                    operation: BuildableUiOperationType::Scroll { 
                        scroll: BuildableScrollOperation { 
                            scroll_to: BuildableScrollOperationType::Active {
                                include_children: true
                            } 
                        }
                    }.into()
                }
            })
        );
    }

}