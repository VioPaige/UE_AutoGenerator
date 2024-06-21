use std::{
    fs::{
        read_dir,
        metadata
    },
    path::PathBuf,
};

#[allow(dead_code)]
struct TreeInfo {
    size: u64,
    hasDirs: bool,
    hasFiles: bool,
}

#[allow(dead_code)]
pub struct DirInfo {
    path: PathBuf,
    shallow: bool,
    size: u64,
}

#[allow(dead_code)]
fn getTreeSize(dir: PathBuf) -> TreeInfo {
    let mut info = TreeInfo {
        size: 0,
        hasDirs: false,
        hasFiles: false
    };

    let dir = read_dir(dir).expect("Something went wrong while reading directory.");
    for entry in dir {
        let path = entry.expect("Something went wrong while reading a file.").path();
        let mdata = metadata(&path).expect("Something went wrong while getting metadata of a file.");
        
        if mdata.is_dir() {
            info.hasDirs = true;
            info.size +=getTreeSize(path).size;
        } else if mdata.is_file() {
            info.hasFiles = true;
            info.size +=1;
        }
    }

    return info;
}

pub fn getRecursiveJsonFilesFromDir(dir: PathBuf) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();

    let dirObject = read_dir(dir).expect("Something went wrong while reading directory.");
    for entry in dirObject {
        let path = entry.expect("Something went wrong while reading a file.").path();
        let mdata = metadata(&path).expect("Something went wrong while getting metadata of a file.");

        if mdata.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
            files.push(path);
        } else if mdata.is_dir() {
            let innerFiles = getRecursiveJsonFilesFromDir(path);
            for file in innerFiles { files.push(file) }
        }
    }

    return files;
}

#[allow(dead_code)]
pub fn getDirsWithMaxSize(dir: PathBuf, maxBatchSize: u64) -> Vec<DirInfo> {
    let mut dirs: Vec<DirInfo> = Vec::new();

    let dirObject = read_dir(&dir).expect("Something went wrong while reading directory.");
    for entry in dirObject {
        let path = entry.expect("Something went wrong while reading a file.").path();
        let mdata = metadata(&path).expect("Something went wrong while getting metadata of a file.");

        if !mdata.is_dir() { return dirs }
        let info = getTreeSize(path.clone());

        if info.size < maxBatchSize || !info.hasDirs {
            dirs.push(DirInfo {
                path,
                shallow: false,
                size: info.size,
            });
        } else {
            if info.hasFiles {
                dirs.push(DirInfo {
                    path: path.clone(),
                    shallow: true,
                    size: info.size,
                });
            }

            let dirInfo = getDirsWithMaxSize(path, maxBatchSize);
            for di in dirInfo { dirs.push(di) }
        }
    }    

    return dirs;
}