#[macro_use]
pub mod common;

pub mod commands;

pub mod problem;
pub mod solution;
pub mod util;

pub mod prelude {
    pub use crate::commands::init::InitCommand;
    pub use crate::commands::pull::PullCommand;
    pub use crate::consts::TITLE_TEXT;
}

pub mod consts {
    /// https://patorjk.com/software/taag/#p=display&f=Whimsy&t=QuipCode
    pub const TITLE_TEXT: &str = r"
                     d8,                               d8b        
                    `8P                                88P        
                                                      d88         
.d88b,.88P?88   d8P  88b?88,.d88b, d8888b d8888b  d888888   d8888b
88P  `88P'd88   88   88P`?88'  ?88d8P' `Pd8P' ?88d8P' ?88  d8b_,dP
?8b  d88  ?8(  d88  d88   88b  d8P88b    88b  d8888b  ,88b 88b    
`?888888  `?88P'?8bd88'   888888P'`?888P'`?8888P'`?88P'`88b`?888P'
    `?88                  88P'                                    
      88b                d88                                      
      ?8P                ?8P                                      
";
}
