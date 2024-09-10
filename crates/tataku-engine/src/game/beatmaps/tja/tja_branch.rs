use crate::prelude::*;

/// a group of branches which start at the same time
#[derive(Clone, Debug)]
pub struct TjaBranchGroup {
    pub start_time: f32,
    pub requirement: BranchRequirement,
    pub branches: HashMap<BranchDifficulty, TjaBranch>,
}

#[derive(Clone, Default, Debug)]
/// a TJA branch
pub struct TjaBranch {
    pub diff: BranchDifficulty,
    pub circles: Vec<TjaCircle>,
    pub drumrolls: Vec<TjaDrumroll>,
    pub balloons: Vec<TjaBalloon>,
}



#[derive(Copy, Clone, Debug, Default)]
/// the requirements for the branches
pub struct BranchRequirement {
    /// what is the requirement type for this branch?
    pub requirement_type: BranchRequirementType,

    /// requirement for the advanced path
    pub advanced: f32,
    
    /// requirement for the master branch
    pub master: f32,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
/// a branch's requirement type
pub enum BranchRequirementType {
    #[default]
    Drumroll,
    Accuracy,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
/// the difficulty type for a branch
pub enum BranchDifficulty {
    #[default]
    Normal,
    Advanced,
    Master,
}