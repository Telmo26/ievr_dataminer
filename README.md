# IEVR Dataminer

This tool extracts structured game data from *Inazuma Eleven Victory Road* into standard SQLite databases for analysis, tooling, or modding.

Relies on [IEVR Toolbox](https://github.com/Telmo26/ievr_toolbox) for extracting the encrypted, compressed game files.

# Features

- **Character data**: This tool automatically extracts element, main position, alternative position, archetype, and stats for every character. See roadmap section for future data to be extracted.
- **Translation data**: This tool is language agnostic. The character data extracted is only comprised of numbers, and every language file is extracted in its own database, so that everyone can use this tool, regardless of what language they actually want to display the extracted data in.

# Requirements

This project is written in Rust, therefore the compiled binaries are static: there are no external requirements. The program is only available for Linux and Windows as of now, but if there is demand I could try to compile a MacOS version using GitHub Actions.

# Usage

The program doesn't have an interface: it is terminal only. However, since there are no command line arguments to be passed, you can use it by simply double clicking on it (or on Linux running it from a terminal, depending on your desktop environment).

You only need to download the executable file from the [latest release](https://github.com/Telmo26/ievr_dataminer/releases). On first startup, the program will download the  `settings.toml` file from this repository, and ask you to fill it. There are default values in it, but if you want to extract game files you will need to fill in the game's path.

Once the file is correctly filled, starting the program again will extract only the relevant game files thanks to [IEVR Toolbox](https://github.com/Telmo26/ievr_toolbox), and then parse them in parallel into databases in the `output` directory. The databases will be named as follows:
- `characters.sqlite`: contains all the information about characters, heroes and basaras. The characters' names will be unique integer identifiers. You then have to go through the text database corresponding to your language to get the character's name.
- `text/{language}.sqlite`: contains all the text information relevant to the extracted data. There will be one database per language (so `en.sqlite`, `ja.sqlite`...), so that you can go and get the translation you need.

For detailed documentation of the database structure and example queries, see the GitHub Wiki.

# Roadmap

- [ ] Fix level 99 stats calculation
- [ ] Extract skill data and link it to characters
- [ ] Extract face PNGs