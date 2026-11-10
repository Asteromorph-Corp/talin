fn skills() -> ::talin::skill::SkillSet {
    let mut skills = ::talin::skill::SkillSet::new();
    skills.insert(::talin::skill::FsSkill::in_home("pdf"));
    skills.insert(::talin::skill::FsSkill::in_home("xlsx"));
    skills
}
