fn main() {
    #[cfg(feature = "bindings")]
        {
            use ligen::prelude::*;
            use ligen_cargo::{CargoProject, CargoGenerator};
            use ligen_rust::RustGenerator;
            use ligen_c::CGenerator;
            use ligen_cmake::{CMakeGenerator, Language};

            match CargoProject::current().and_then(Project::try_from) {
                Ok(project) => {
                    CargoGenerator::default().generate(&project).expect("Failed to generate Cargo interface.");
                    RustGenerator::default().generate(&project).expect("Failed to generate Rust interface.");
                    CGenerator::default().generate(&project).expect("Failed to generate C interface.");
                    CMakeGenerator(Language::C).generate(&project).expect("Failed to generate CMake project.");
                    // CSharpGenerator::default().generate(&project).expect("Failed to generate C# interface.");
                    // CargoBuilder.build(&project, BuildProfile::Release).expect("Failed to build Cargo project.");
                },
                Err(_) => ()
            }
        }
}