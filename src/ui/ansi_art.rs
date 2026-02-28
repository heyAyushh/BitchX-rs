use rand::Rng;

/// ANSI art logo variant 0 - Diamond pattern with colored BitchX text
/// Ported from original C source (source/art.c case 0)
pub const LOGO_DIAMOND: &str = concat!(
    "\x1b[40m\n",
    "::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::\n",
    ":::\x1b[1;31m:.\x1b[0m.\x1b[1;31m.\x1b[14C\x1b[0m.\x1b[1;35m.:\x1b[0m:::::::::::::::::\x1b[1;31m:.\x1b[0m.\x1b[14C\x1b[1;35m.\x1b[0m.\x1b[1;35m.:\x1b[0m:::::::::::::::::\n",
    ":::::::\x1b[1;31m:.\x1b[11C\x1b[35m:\x1b[0m:::::::::::::::::::::::\x1b[1;31m:\x1b[11C\x1b[35m.:\x1b[0m:::::::::::::::::::::\n",
    ":::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m:::::::::::::::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m:::\x1b[1;31m:\"\x1b[0m`````````````````\"\n",
    ":::::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m:::::::::::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m::::\x1b[1;31m: \x1b[35mB\x1b[0;35mitch\x1b[1mX \x1b[0mby \x1b[1;32mp\x1b[0;32manasync\x1b[36m!\n",
    "\x1b[37m:::::::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m:::::::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m:::::::\x1b[1;31m:.\x1b[0m..................\n",
    ":::::::::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m:::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m:::::::::::::::::::::::::::::\n",
    "::\x1b[1;31m:\"\x1b[0m\"````\"\x1b[1;35m\":\x1b[0m::\x1b[1;31m:'\x1b[36m.g$\x1b[0;36m$S\x1b[32m$'\x1b[6C\x1b[1;35m`:\x1b[0m:::\x1b[1;31m:'\x1b[11C\x1b[35m\":\x1b[0m::\x1b[1;31m:\"\x1b[0m\"```\x1b[1;35m\":\x1b[0m::::::::::::::::::::\n",
    "\x1b[1;31m'\x1b[36ms#S\x1b[0;36m$$$\"\x1b[1m$$\x1b[0;36mS#\x1b[32mn.\x1b[1;31m` \x1b[36m$$\x1b[0;36m$$\x1b[32mS\". \x1b[1;36ms#S\x1b[0;36m$$\x1b[32m$ \x1b[1;35m`\x1b[31m:'   \x1b[36m.g#\x1b[0;36mS$$\x1b[1m\"$\x1b[0;36m$S#\x1b[32mn. \x1b[1;36ms#S\x1b[0;36m$$\x1b[32m$ \x1b[1;35m`\x1b[0m\"\x1b[1;35m\":\x1b[0m::::::::::::::::\n",
    " \x1b[1;36m$$\x1b[0;36m$$$\x1b[32m$\x1b[1;36m_,$\x1b[0;36m$\x1b[32m$S'\x1b[1;30mrE\x1b[36m.g#\x1b[0;36mS$$\x1b[32m$ \x1b[1;36m$$\x1b[0;36m$$$$ss\x1b[32mn    \x1b[1;36m$$\x1b[0;36m$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$\x1b[32m$$$ \x1b[1;36m$$\x1b[0;36m$$$$\x1b[1m\"$\x1b[0;36m$S#\x1b[32mn.\x1b[1;35m`:\x1b[0m::::::::::::\n",
    " \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$\x1b[1;36m`\"$\x1b[0;36m$SSn\x1b[32m. \x1b[1;36m$$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36mgg#\x1b[0;36mS\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36mggg\x1b[0;36mgg\x1b[32mn \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;35m:\x1b[0m:\x1b[1;31m::\"\x1b[0m````````\n",
    " \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$  \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$$\x1b[0;36m$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$$\x1b[0;36m$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;35m:\x1b[0m:\x1b[1;31m: \x1b[37mG\x1b[0mreets \x1b[1mT\x1b[0mo\x1b[1;30m\n",
    " \x1b[36m$\x1b[0;36m$$$$\x1b[32m$  \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;35m:\x1b[0m:\x1b[1;31m: \x1b[36mT\x1b[0;36mrench\x1b[1;30m,\n",
    " \x1b[36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m,$\x1b[0;36m$$$\x1b[32m$$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$\x1b[32m$$ \x1b[1;36m$\x1b[0;36m$$$$\x1b[32m$ \x1b[1;36m$\x1b[0;36m$$$\x1b[32m$$ \x1b[1;36m$\x1b[0;36m$$$\x1b[32m$$ \x1b[1;36m$\x1b[0;36m$$$\x1b[32m$$ \x1b[1;35m:\x1b[0m:\x1b[1;31m: \x1b[36mL\x1b[0;36mifendel\x1b[1;30m,\n",
    " \x1b[36m$\x1b[0;36m$$$$$ss$$$\x1b[32m$S' \x1b[1;36m$\x1b[0;36m$$$\x1b[32m$$$ \x1b[1;36m`S\x1b[0;36m$$$$s$$\x1b[32m$S' \x1b[1;36m`S\x1b[0;36m$$$$s$$$\x1b[32m$S' \x1b[1;36m$\x1b[0;36m$$\x1b[32m$$$ \x1b[1;36m$\x1b[0;36m$$\x1b[32m$$$ \x1b[1;35m:\x1b[0m:\x1b[1;31m: \x1b[36mJ\x1b[0;36mondala\x1b[1mR\x1b[30m,\n",
    "\x1b[31m.\x1b[0m............\x1b[1;35m.:\x1b[31m:\x1b[11C\x1b[35m.\x1b[0m......\x1b[1;35m.:\x1b[31m:.\x1b[11C\x1b[35m:\x1b[31m:.\x1b[0m.....\x1b[1;31m:\x1b[0m.......\x1b[1;35m:\x1b[0m:\x1b[1;31m: \x1b[36mZ\x1b[0;36mircon\x1b[1;30m,\n",
    "\x1b[0m:::::::::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m:::::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m::::::\x1b[1;31m:\"\x1b[0m``````\x1b[1;31m`' \x1b[36mO\x1b[0;36mtiluke\x1b[1;30m,\n",
    "\x1b[0m:::::::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m:::::::::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m:::\x1b[1;31m: \x1b[36mH\x1b[0;36mappy\x1b[1mC\x1b[0;36mrappy\x1b[1;30m, \x1b[36mY\x1b[0;36mak\x1b[1;30m,\n",
    "\x1b[0m:::::::::\x1b[1;31m:'\x1b[10C\x1b[35m.:\x1b[0m:::::::::::::::::::\x1b[1;31m:.\x1b[10C\x1b[35m`:\x1b[0m:\x1b[1;31m: \x1b[36mM\x1b[0;36masonry\x1b[1;30m, \x1b[36mB\x1b[0;36muddha\x1b[1mX\x1b[30m..\n",
    "\x1b[0m:::::::\x1b[1;31m:'\x1b[11C\x1b[35m:\x1b[0m:::::::::::::::::::::::\x1b[1;31m:\x1b[11C\x1b[35m`:\x1b[31m:.\x1b[0m...................\n",
    ":::\x1b[1;31m:\"\x1b[0m\"\x1b[1;31m'\x1b[14C\x1b[0m`\x1b[1;35m\":\x1b[0m:::::::::::::::::\x1b[1;31m:\"\x1b[0m'\x1b[14C\x1b[1;35m`\x1b[0m\"\x1b[1;35m\":\x1b[0m:::::::::::::::::\n",
    "::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::\x1b[0m\n",
    "\x1b[0m\n",
);

/// ASCII art logo variant - acidjazz style (case 2)
/// No ANSI color codes, most portable variant
pub const LOGO_ACIDJAZZ: &str = concat!(
    "\x1b[40m\n",
    "                                                                   ,\n",
    "                                           .                     ,$\n",
    "                 .                                              ,$'\n",
    "                                           .        .          ,$'\n",
    "                 :      ,g$p,              .         $,       ,$'\n",
    "               y&$       `\"` .,.           $&y       `$,     ,$'\n",
    "               $$$     o oooy$$$yoo o      $$$        `$,   ,$' -acidjazz\n",
    "         .     $$$%%yyyp, gyp`$$$'gyyyyyyp, $$$yyyyp,   `$, ,$'     .\n",
    "       . yxxxx $$$\"`\"$$$ $$$ $$$ $y$\"`\"$$$ $$$\"`\"$$$ xxx`$,$'xxxxxxy .\n",
    "         $     $$7   l$$ $$$ $$$ $$7   \"\"\" $$7   ly$     .$'       $\n",
    "         $     $$b   dy$ $$$ $y$ $$b   $$$ $$b   d$$    ,$`$,      $\n",
    "       . $xxxx $$$uuu$$$ $$$ $$$ $$$uuu$$$ $$$   $$$ x ,$'x`$, xxxx$ .\n",
    "         .           \"\"\" \"\"\" \"\"\"       \"\"\"       \"\"\"  ,$'   `$,    .\n",
    "           b i t c h    -      x                     ,$'     `$,\n",
    "                                                     $'       `$,\n",
    "                                                    '          `$,\n",
    "                                                                `$,\n",
    "                                                                 `$\n",
    "                                                                   `\n",
    "\x1b[0m\n",
);

/// ANSI art logo variant 14 - Minimal underline style
/// Ported from original C source (source/art.c case 14)
pub const LOGO_MINIMAL: &str = concat!(
    "\x1b[40m\n",
    "\x1b[8C\x1b[1;35m________\x1b[0m\x1b[9C\x1b[1;30m   \x1b[35m________ \x1b[0m\x1b[8C\x1b[1;35m________\x1b[30m \x1b[0m   \x1b[1;30m \x1b[0m\x1b[4C\x1b[1;35m________\n",
    "\x1b[0m\x1b[8C\x1b[1;35m\\\x1b[0m\x1b[6C\x1b[1;35m//___________\\\x1b[0m\x1b[6C\x1b[1;35m/________\\\\\x1b[0m\x1b[6C\x1b[1;35m/_________\\_\x1b[0m\x1b[5C\x1b[1;35m//\n",
    "\x1b[0m\x1b[6C\x1b[1;35m___\x1b[0;35m\\\x1b[37m \x1b[1;30m \x1b[0m \x1b[1;30m \x1b[0;35m___\x1b[37m   \x1b[35m_________\x1b[1;30m \x1b[0m\x1b[4C\x1b[35m__\x1b[37m\x1b[5C\x1b[35m_______\x1b[37m\x1b[8C\x1b[1;30m \x1b[0;35m\\\x1b[37m\x1b[5C\x1b[35m/\x1b[37m\x1b[4C\x1b[35m/\n",
    "\x1b[37m\x1b[8C\x1b[35m<<_____\x1b[1;30m \x1b[0m\x1b[4C\x1b[35m\\\\\x1b[1;30m \x1b[0m\x1b[4C\x1b[1;30m \x1b[0;35m/\x1b[37m\x1b[6C\x1b[35m> \x1b[37m\x1b[4C\x1b[35m\\\x1b[37m   \x1b[1;30m \x1b[0m \x1b[35m/____\\\x1b[1;30m  \x1b[0m   \x1b[35m>>\x1b[37m\x1b[8C\x1b[35m\\\x1b[1;35m ___\n",
    "\x1b[0m\x1b[6C\x1b[1;30m____\x1b[0m\x1b[4C\x1b[1;30m/______\\_____<<_____//___________>>  /_______\\\x1b[0m   \x1b[1;30m/_____>>sm\n",
    "\x1b[0m\x1b[8C\x1b[1;30m<<___________\x1b[0m\x1b[7C\x1b[1;30m  \x1b[0m bitchx by panasync\x1b[7C\x1b[1;30m /______\\\\\x1b[0m   \x1b[1;30m____\n",
    "\x1b[0m\x1b[20C\x1b[1;30m/------------------------------------------------\\\\\n",
    "\n",
    "\n",
    "\x1b[0m\n",
);

const LOGOS: &[&str] = &[LOGO_DIAMOND, LOGO_ACIDJAZZ, LOGO_MINIMAL];

/// Print a randomly selected ANSI art logo
pub fn print_ansi_logo() {
    let mut rng = rand::thread_rng();
    let idx = rng.gen_range(0..LOGOS.len());
    print!("{}", LOGOS[idx]);
}

/// Print startup banner with a random logo and the version tagline
pub fn print_startup_banner() {
    print_ansi_logo();
    println!("  BitchX 2.0 - Rust Rewrite");
    println!("  Type /help for commands");
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_ansi_logo_does_not_panic() {
        print_ansi_logo();
    }

    #[test]
    fn acidjazz_contains_bitchx() {
        let lower = LOGO_ACIDJAZZ.to_lowercase();
        assert!(
            lower.contains("bitchx") || lower.contains("b i t c h"),
            "acidjazz logo should contain bitchx or 'b i t c h'"
        );
    }

    #[test]
    fn diamond_contains_ansi_escapes() {
        assert!(
            LOGO_DIAMOND.contains("\x1b["),
            "diamond logo should contain ANSI escape sequences"
        );
    }

    #[test]
    fn minimal_contains_bitchx() {
        let lower = LOGO_MINIMAL.to_lowercase();
        assert!(
            lower.contains("bitchx"),
            "minimal logo should contain bitchx"
        );
    }

    #[test]
    fn logos_array_has_three_variants() {
        assert_eq!(LOGOS.len(), 3);
    }
}
