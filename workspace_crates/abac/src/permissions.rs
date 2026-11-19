#[derive(strum::Display, Debug, Clone, Copy, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum Permission {
    SetCustomThreadIdWhenCreatingIt,
    ManageCustomSitePages,
    CanCreateForum,
    CanCreateThread,
    ViewAppExecutableVersion,
    ManageCustomPageStylesheets,
    CanEditSiteVariables,
    CanViewSiteVariables,
    Superuser,
}
