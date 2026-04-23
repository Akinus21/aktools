pub fn help() -> i32 {
    println!(r#"AKTools - Modular CLI Tool Runner

AKTools lets you turn any script into a modular CLI command with
custom aliases and multiple entry points. Manage all your custom
commands in one place.

USAGE:
    aktools <command> [options]

COMMANDS:
    add <filename>     Add a script as a module. Prompts for name
                      and aliases. Supports multiple command flags.

    edit [module]      Edit a module's manifest. Add flags, change
                      aliases, modify commands. Interactive menu.

    list              Show all installed modules with their aliases.

    rm <module>       Remove a module and its files.

    update            Rebuild registry from module folders.

    doctor            Diagnose and auto-fix configuration issues.
                      Creates directories, fixes shell integration.

    help              Show this help message.

MODULE STRUCTURE:
    Modules live in ~/.aktools/modules/
    Each module is a folder with:
      - manifest.xml    Module metadata
      - Scripts and resources

MANIFEST EXAMPLE:
    <?xml version="1.0"?>
    <module>
        <name>MyScript</name>
        <alias>ms</alias>
        <option>
            <flag>*run</flag>      # * = default flag
            <command>./run.sh</command>
        </option>
        <option>
            <flag>list</flag>
            <command>./list.sh</command>
        </option>
    </module>

QUICK START:
    aktools add myscript.sh
    aktools list
    aktools doctor

For more details, see the README at:
https://github.com/Akinus21/aktools
"#);
    0
}