extern crate regex;

use super::{
    config::Config,
    fs::getRecursiveJsonFilesFromDir,
    output::colouriseLog,
};
use std::{
    collections::HashMap, 
    fs::{
        canonicalize,
        create_dir_all,
        metadata,
        read_dir, 
        remove_file, 
        rename, 
        File
    }, 
    io::Read, 
    path::{
        Path, 
        PathBuf,
    }, 
    process::Command, 
    u32
};
use self::regex::Regex;



pub fn lookForMatch(config: Config, parsedArgs: HashMap<String, String>) {
    let matchString: String = parsedArgs.get("match").expect("The match argument is required for lookformatch (lfm) process execution.").to_string();
    let files = getRecursiveJsonFilesFromDir(config.fileConfig.nongendump.expect("Nongendump path in configuration file is invalid."));
    let mut matches: Vec<String> = Vec::new();

    colouriseLog(format!("Got full file list (<<green>>{} files</green>>)", files.len()), false);

    for (index, path) in files.iter().enumerate() {
        let fileName = path.file_name().expect("Couldn't get file name").to_str().expect("Couldn't get file name");
        colouriseLog(format!("Checking file <<grey>>{}</grey>> (<<green>>{}/{}</green>>)", fileName, index + 1, files.len()), true);

        let mut fileContents = String::new();
        File::open(path).expect("Something went wrong while reading a file.").read_to_string(&mut fileContents).expect("Something went wrong while reading a file.");
        
        if fileContents.to_lowercase().contains(&matchString.to_lowercase()) {
            matches.push(path.to_str().expect("Failed to get matching file path as string.").to_string());
        }
    }

    colouriseLog(format!("\nFound matches in the following files:\n<<green>>{}</green>>", matches.join("\n")), false);
}

struct FileIterationInfo {
    rootPath: PathBuf,
    modPath: PathBuf, 
    iteration: u32,
    assetType: String,
}
fn recursiveGetDependencies(filePath: String, iteration: u32, highestIteration: &mut u32, silentMode: bool) -> Vec<FileIterationInfo> {
    *highestIteration = iteration;

    let mut files: Vec<FileIterationInfo> = Vec::new();

    let mut fileContents = String::new();
    if !silentMode { println!("Reading file: {:?}", &filePath); }
    File::open(&filePath).expect("Couldn't find target file.").read_to_string(&mut fileContents).expect(format!("Failed to read target file contents. {}", &filePath).as_str());

    let regexMatcher1 = "AssetClass\".*?\"(.*?)\"";
    let re1 = Regex::new(&regexMatcher1).unwrap();

    let assetType: String = re1.captures_iter(&fileContents).next().map(|caps| caps.get(1).unwrap().as_str()).unwrap_or_default().to_string();

    files.push(FileIterationInfo {
        rootPath: PathBuf::from(&filePath),
        modPath: PathBuf::from(&filePath.replace("nongenout/", "out/")),
        iteration,
        assetType,
    });

    let regexMatcher2 = "(\\/Game.*?)[\"|\\.]";
    let re2 = Regex::new(&regexMatcher2).unwrap();

    let mut alreadyFound: Vec<String> = Vec::new();
    for capture in re2.captures_iter(&fileContents) {

        for (i, cap) in capture.iter().enumerate() {
            if i == 0 { continue }
            
            let mut c = format!("nongenout{}", cap.expect("Failed to gather matched text.").as_str().to_string());
            if alreadyFound.contains(&c) { continue }
            alreadyFound.push(c.clone());

            if filePath.replace("\\", "/").contains(&c) { continue }
            if !c.ends_with(".json") { c.push_str(".json") }

            let deps = recursiveGetDependencies(c, iteration + 1, highestIteration, silentMode);
            for dep in deps { files.push(dep) }
        }
    }

    return files
}

pub fn generateTarget(config: Config, parsedArgs: HashMap<String, String>) {
    println!("GenerateTarget process started.");

    let silentMode = parsedArgs.get("silent").is_some();
    let path: String;
    if let Some(p) = parsedArgs.get("path") { path = p.to_string() } else { return }
        
    let mut highestIteration = 0;
    let mut files: Vec<FileIterationInfo> = Vec::new();

    let mode = parsedArgs.get("mode").unwrap();
    if mode == "targeted" || mode == "t" {
        files = recursiveGetDependencies(path, 0, &mut highestIteration, silentMode);
    } else {
        let targets = read_dir(&path).expect("Failed to read target directory.");
        for target in targets {
            let mdata = metadata(target.as_ref().expect("Failed to read target file.").path());
            if mdata.unwrap().is_dir() { continue }

            let filename = target.unwrap().file_name();
            let deps = recursiveGetDependencies(format!("{}/{}", path, filename.to_str().unwrap()), 0, &mut highestIteration, silentMode);
            for dep in deps { files.push(dep) }
        }
    }

    colouriseLog(format!("Finished with highest iteration <<green>>{}</green>> and <<green>>{}</green>> file{}.", highestIteration, files.len(), if files.len() == 1 { "" } else { "s" }), false);

    let mut fails: Vec<String> = Vec::new();
    let mut successes: Vec<String> = Vec::new();
    for i in (0..=highestIteration).rev() {
        let targets: Vec<&FileIterationInfo> = files.iter().filter(|fileInfo| fileInfo.iteration == i).collect();
        let mut foundTypes: Vec<String> = Vec::new();

        colouriseLog(format!("Starting iteration {} with {} files. <<grey>>({} to go)</grey>>", i, targets.len(), i), false);

        for target in &targets {
            if !Path::new(&target.rootPath).exists() { continue }

            create_dir_all(&target.modPath.parent().expect("Failed to find parent path.")).expect("Failed to create parent directories.");
            rename(&target.rootPath, &target.modPath).expect(&format!("Failed to move file '{:?}'", target.rootPath));
        }

        for target in &targets {
            let pureAssetPath = target.rootPath.to_str().expect("Failed to transform path to str.").replace(
                config.fileConfig.nongendump.clone().expect("Failed to get nongendump.").to_str().expect("Failed to transform nongendump path to str."), 
                ""
            ).replace(".json", ".uasset").replace("\\Game\\", "/Content/").replace("/Game/", "Content/");

            let mut projectAssetPath = config.fileConfig.projectdir.clone().expect("Failed to load project directory path.");
            projectAssetPath.push(pureAssetPath);

            let refreshExisting = config.fileConfig.refreshExistingAssets.clone().expect("Failed to load refreshExistingAssets value.");
            if Path::new(&projectAssetPath).exists() && refreshExisting {
                remove_file(&projectAssetPath).expect("Failed to delete previous version of generated asset.");

                if !silentMode { colouriseLog(format!("<<red>>Removed</red>> {:?}", projectAssetPath), false); }
            } else if refreshExisting && !silentMode {
                colouriseLog(format!("<<red>>Removed</red>> {:?} <<grey>>(did not exist)</grey>>", projectAssetPath), false);
            }
        }

        for target in &targets {
            if !foundTypes.contains(&target.assetType) {
                foundTypes.push(target.assetType.clone());
            }
        }

        // sorts depending on which asset type is first in the config file.. is a bit of a mess? dunno if this can be done better, just did something and then did all the debug suggestions until no more red lines :3
        foundTypes.sort_by(|a, b| {
            let r = config.fileConfig.assetOrder.as_ref();

            r.expect("Failed to load assetOrder.").iter().position(|x| x == a).unwrap().cmp(&r.expect("Failed to load assetOrder").iter().position(|x| x == b).unwrap())
        });

        colouriseLog(format!("<<grey>>Found Types</grey>>: {}", &foundTypes.join(", ")), false);

        for (index, assetType) in foundTypes.iter().enumerate() {
            let commandString = format!(
                "{} \"{}\" -run=AssetGenerator -DumpDirectory={:#?} -AssetClassWhitelist=\"{}\" abslog=\"D:/UE5/Projects/PaydayAssetGen/MoolahProject-main/Manual/logs/genlog.txt\" -stdout -unattended {}",

                config.fileConfig.ue4cmd.clone().expect("Failed to load ue4cmd path.").to_str().expect("Failed to transform ue4cmd path to str."),
                config.fileConfig.project.clone().expect("Failed to load project path.").to_str().expect("Failed to transform project path to str."),
                canonicalize(config.fileConfig.dumpdir.clone().expect("Failed to load dumpdir path.")).expect("Failed to canonicalise.").to_str().unwrap().replace("\\", "/").replace("//?/", ""),
                assetType,
                if config.fileConfig.refreshExistingAssets.unwrap() { "" } else { "-NoRefresh" }
            );
            let mut command = commandString.split_whitespace();
            let cmd = command.next();
            let args = command.collect::<Vec<&str>>();

            if !silentMode { println!("Running command: {} - with args: {}", cmd.unwrap(), args.join(" ")); }
            

            match Command::new(cmd.unwrap().to_string()).args(args).output() {
                Ok(output) => {
                    if !output.stderr.is_empty() {
                        println!("<<red>>{}:</red>>\n{}", assetType, String::from_utf8_lossy(&output.stderr));
                    }

                    colouriseLog(format!(
                        "<<grey>>Finished with iteration {} with AssetType</grey>> {} <<grey>>({}/{})</grey>>",
                        i,
                        assetType,
                        index + 1,  
                        foundTypes.len()
                    ), false);
                },
                Err(e) => {
                    colouriseLog(format!("<<grey>>Iteration {} with AssetType {}:</grey>>\n<<red>>{}</red>>", i, assetType, e), false);
                }
            }

            for target in &targets {
                if !Path::new(&target.modPath).exists() { continue }

                rename(&target.modPath, &target.rootPath).expect(&format!("Failed to move file '{:?}'", target.rootPath));
            }
        }

        for target in targets {
            let pureAssetPath = target.rootPath.to_str().expect("Failed to transform path to str.").replace(
                config.fileConfig.nongendump.clone().expect("Failed to get nongendump.").to_str().expect("Failed to transform nongendump path to str."), 
                ""
            ).replace(".json", ".uasset").replace("\\Game\\", "Content/").replace("/Game/", "Content/");

            let mut projectAssetPath = config.fileConfig.projectdir.clone().expect("Failed to load project directory path.");
            projectAssetPath.push(&pureAssetPath);

            if Path::new(&projectAssetPath).exists() {
                successes.push(projectAssetPath.to_str().expect("Failed to transform path to str.").to_string());
            } else {
                fails.push(projectAssetPath.to_str().expect("Failed to transform path to str.").to_string());
            }
        }
    }

    println!("Finished all iterations.");
    let formatString = if fails.len() == 0 { "<<green>>No Fails.</green>>".to_string() } else { format!("<<red>>Failed files</red>>:\n{}", fails.join("\n")) };
    colouriseLog(formatString, false);
}







