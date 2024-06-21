# UE_AutoGenerator
Automated generation of target assets and their dependencies from serialised format.

The CLI tool UE_AutoGenerator (AutoGen) has a set of commands that can be useful in the process of modding unreal engine games, below is the documentation of how to use the CLI.  
It is recommended not to modify the base environment unless you are completely sure of what you are doing, some values such as the names of folders "nongenout" and "out" are hard values.  
This CLI has been built around modding for PAYDAY 3 (UE4.27), and is not guaranteed to work for all games and engine versions, though it hopefully does and I'll be glad to make edits to expand support.

## Dependencies
You need the Unreal Engine CMD executable, as well as a built unreal engine project with the AssetGenerator and AssetDumper plugins.

## Documentation
Before utilising this tool, make sure to set up the config.json file to have the same paths, these should all work with relative as well as absolute paths.  
Also make sure your serialised game assets are located in a "nongenout" directory in the same location as the executable.

Whenever AutoGen is run, it should have its `mode` argument specified, like so:
```
AutoGen.exe mode=targeted
```
The available modes are:
 - "targeted" (t): You specify a *relative* path to the serialised json you want generated, and AutoGen generates it and its dependencies automatically. Example:
    ```
    AutoGen mode=t path=nongenout/Game/UI/Widgets/WBP_UI_ProgressBar.json
    ```
 - "targeteddir" (td): This mode is essentially the same as "targeted", but it generates all assets (and their deps) in a directory. Example:
    ```
    AutoGen mode=td path=nongenout/Game/UI/Widgets
    ```
 - "lookformatch" (lfm): Combs through all files in nongenout directory and gives a list of files in which your match was found (case insensitive). Example:
    ```
    AutoGen mode=lfm match=sbzcookingstation
    ```
For those who like an empty console, there is a `silent` mode. Example:  
```
AutoGen mode=t path=some/path/to/something.json silent=true
```

## Important notes
Recommended to keep refreshing in config.json to false, unless you know for sure that the program won't remove/overwrite any assets you've already edited.  
Credit to Nadz for doing nothing ;)

## Contributions
Feel free to add features, squish bugs, or add wider support in a forked repo and then PR into this one!