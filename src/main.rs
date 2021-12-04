use std::{
  ffi::OsStr,
  fs::rename,
  path::{Path, PathBuf},
};

use id3::Tag;

fn move_to_tagless_folder(path: &Path) {
  eprintln!("tagless file: {path}", path = path.display());
  let old_file_name = match path.file_name() {
    Some(file_name) => file_name,
    None => {
      panic!("was told to move a non-file to the tagless folder: {:?}", path.display());
    }
  };
  if let Err(e) = rename(path, Path::new("D:\\music-tagless").join(old_file_name)) {
    panic!("error while moving a tagless file: {:?}", e);
  }
}

fn main() {
  recursive_read_dir("D:\\music", |path_buf| {
    //println!("Processing file `{path_buf}`...", path_buf = path_buf.display());
    let extension = path_buf.extension().map(OsStr::to_str).flatten();
    match extension {
      Some("mp3") | Some("MP3") => (),
      Some("csv") => {
        if let Err(err) = std::fs::remove_file(&path_buf) {
          panic!(
            "Error while removing `{path_buf}`: {err}",
            path_buf = path_buf.display(),
            err = err
          );
        }
        return;
      }
      _ => return,
    }
    if extension != Some("mp3") && extension != Some("MP3") {
      return;
    }
    let tag = match Tag::read_from_path(&path_buf) {
      Ok(tag) => tag,
      Err(_) => {
        move_to_tagless_folder(path_buf.as_ref());
        return;
      }
    };
    let artist = match tag.artist() {
      Some(artist) => artist
        .replace(":", "-")
        .replace("/", "-")
        .replace("\\", "-")
        .replace("?", "-")
        .replace("\"", "'")
        .replace("<", "[")
        .replace(">", "]")
        .replace("|", "-")
        .replace("*", "-"),
      None => {
        move_to_tagless_folder(path_buf.as_ref());
        return;
      }
    };
    let artist = artist.trim();
    let year = tag.year().unwrap_or(0);
    let album = match tag.album() {
      Some(album) => album
        .replace(":", "-")
        .replace("/", "-")
        .replace("\\", "-")
        .replace("?", "-")
        .replace("\"", "'")
        .replace("<", "[")
        .replace(">", "]")
        .replace("|", "-")
        .replace("*", "-"),
      None => {
        move_to_tagless_folder(path_buf.as_ref());
        return;
      }
    };
    let album = album.trim();
    let disc = tag.disc().unwrap_or(0);
    let total_discs = tag.total_discs().unwrap_or(0);
    let track = tag.track().unwrap_or(0);
    let title = match tag.title() {
      Some(title) => title
        .replace(":", "-")
        .replace("/", "-")
        .replace("\\", "-")
        .replace("?", "-")
        .replace("\"", "'")
        .replace("<", "[")
        .replace(">", "]")
        .replace("|", "-")
        .replace("*", "-"),
      None => match path_buf.file_name() {
        Some(os_str) => format!("{}", Path::new(os_str).display()),
        None => {
          panic!("no filename when trying to make a fake title: {:?}", path_buf.display());
        }
      },
    };
    let title = title.trim();
    let folder_location = Path::new("D:").join("music-sorted").join(artist).join(format!(
      "({year}) {album}",
      year = year,
      album = album
    ));
    if let Err(err) = std::fs::create_dir_all(&folder_location) {
      panic!(
        "{path_buf}\nTried to make `{folder_location}` but could not: {err:?}",
        path_buf = path_buf.display(),
        folder_location = folder_location.display(),
        err = err
      );
    }
    let new_location = folder_location.join(format!(
      "[{disc} of {total_discs}][{track:02}] {title}.mp3",
      disc = disc,
      total_discs = total_discs,
      track = track,
      title = title,
    ));
    println!(
      "{path_buf}\n==> {new_location}\n",
      path_buf = path_buf.display(),
      new_location = new_location.display()
    );
    if let Err(err) = rename(&path_buf, &new_location) {
      panic!(
        "error moving from `{from}` to `{to}`: {err:?}",
        from = path_buf.display(),
        to = new_location.display(),
        err = err
      );
    }
  })
}

/// Recursively walks over the `path` given, which must be a directory.
///
/// Your `op` is passed a [`PathBuf`] for each file found.
///
/// ## Panics
/// * If the path given is not a directory.
pub fn recursive_read_dir(path: impl AsRef<Path>, mut op: impl FnMut(PathBuf)) {
  use std::collections::VecDeque;
  //
  let path = path.as_ref();
  assert!(path.is_dir());
  // Note(Lokathor): Being *literally* recursive can blow out the stack for no
  // reason. Instead, we use a queue based system. Each loop pulls a dir out of
  // the queue and walks it.
  // * If we find a sub-directory that goes into the queue for later.
  // * Files get passed to the `op`
  // * Symlinks we check if they point to a Dir or File and act accordingly.
  //
  // REMINDER: if a symlink makes a loop on the file system then this will trap
  // us in an endless loop. That's the user's fault!
  let mut path_q = VecDeque::new();
  path_q.push_back(PathBuf::from(path));
  while let Some(path_buf) = path_q.pop_front() {
    match std::fs::read_dir(&path_buf) {
      Err(e) => eprintln!("Can't read_dir {path}: {e}", path = path_buf.display(), e = e),
      Ok(read_dir) => {
        for result_dir_entry in read_dir {
          match result_dir_entry {
            Err(e) => eprintln!("Error with dir entry: {e}", e = e),
            Ok(dir_entry) => match dir_entry.file_type() {
              Ok(ft) if ft.is_dir() => path_q.push_back(dir_entry.path()),
              Ok(ft) if ft.is_file() => op(dir_entry.path()),
              Ok(ft) if ft.is_symlink() => match dir_entry.metadata() {
                Ok(metadata) if metadata.is_dir() => path_q.push_back(dir_entry.path()),
                Ok(metadata) if metadata.is_file() => op(dir_entry.path()),
                Err(e) => eprintln!(
                  "Can't get metadata for symlink {path}: {e}",
                  path = dir_entry.path().display(),
                  e = e
                ),
                _ => eprintln!(
                  "Found symlink {path} but it's not a file or a directory.",
                  path = dir_entry.path().display()
                ),
              },
              Err(e) => eprintln!(
                "Can't get file type of {path}: {e}",
                path = dir_entry.path().display(),
                e = e
              ),
              _ => eprintln!(
                "Found dir_entry {path} but it's not a file, directory, or symlink.",
                path = dir_entry.path().display()
              ),
            },
          }
        }
      }
    }
  }
}
