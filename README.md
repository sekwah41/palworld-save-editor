# Palworld save editor

This is just a quick project I created to mess around with the palworld saves as I wanted to save players progress after I had reverted to a backup.

Initially this will just be a simple save editor, but I may add more features in the future e.g. copying inventory from a backup file, converting a single player world to a dedicated etc.

This is meant to be a rust version with a GUI inspired by [palworld-save-tools](https://github.com/cheahjs/palworld-save-tools).

The goal is to allow a small installation size with no additional install requirements.
While technically this could be a portable file due to WebView2 being installed on window's 11 it's a tiny install anyway.

For now once its working I will upload a copy manually to the releases page, but I will automate it if I continue to work on this.


Oh and also this is my first svelte project because I wanted to try it out, so I'm sure there are better ways to do things :)

As of creating the project it can only open player saves, check [this ticket](https://github.com/sekwah41/palworld-save-editor/issues/2) to see the progress on opening the other sav files.
