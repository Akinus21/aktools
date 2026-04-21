pub fn execute() -> i32 {
    println!(r#"AKTools - Modular CLI Tool Runner

USAGE:
    aktools <command> [options]

COMMANDS:
    add <filename>     Add a file as a new module. Prompts for name, aliases.
                      Checks for alias conflicts before creating.
                      
    edit [module]     Edit a module's manifest. If no module specified,
                      shows list to choose from. Loops until 'q'.
                      
    rm [module]       Remove a module. If no module specified, shows
                      list to choose from. Loops until 'q'.
                      
    update            Rebuild the registry.json from module folders
                      and their manifest.xml files.
                      
    doctor            Diagnose AKTools issues:
                      - Check module directories exist
                      - Verify shell integration
                      - Validate alias files
                      - Check for updates
                      - Verify module integrity
                      
    help              Show this detailed help message

MODULE STRUCTURE:
    Modules are stored in ~/.aktools/modules/
    Each module is a folder containing:
    
    - manifest.xml     Module metadata in XML format
    - Any scripts or resources the module needs
    
MANIFEST.XML FORMAT:
    <?xml version="1.0"?>
    <module>
        <name>ModuleName</name>
        <alias>alias1</alias>
        <alias>alias2</alias>
        <option>
            <flag>*defaultflag</flag>    (* = default command)
            <command>script to run</command>
        </option>
        <option>
            <flag>otherflag</flag>
            <command>another script</command>
        </option>
    </module>

BUILT-IN OPTIONS:
    add <filename>     Adds file as module. Prompts user for name, aliases.
                      Checks aliases against current aliases, prompts on conflict.
                      Creates module folder, copies file, creates manifest
                      with "Star" option entry (default command).
                      
    edit [blank|module]  Edit module manifest. If blank, shows alphabetical
                        list of installed modules, then runs edit guide.
                        Loops until user chooses 'q'.
                        
    rm/remove [blank|module]  Delete module. If blank, shows alphabetical
                              list. Loops until 'q'.
                              
    update             Recreates project registry from module folders.
    
    doctor             Diagnoses AKTools issues:
                      - Ensures aliases from modules are in shell source
                      - Checks for updates, prompts to install
                      - Fixes any found issues
                      
    help               Shows this detailed help

EXAMPLES:
    aktools add myscript.sh
    aktools edit
    aktools rm
    aktools update
    aktools doctor
    aktools help
"#);
    0
}