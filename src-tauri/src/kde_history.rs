use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::Path;

const DATABASE: &str = ".local/share/kactivitymanagerd/resources/database";
const APPLICATION_PREFIX: &str = "applications:";

pub fn last_launches(home: &Path) -> HashMap<String, i64> {
    read(&home.join(DATABASE)).unwrap_or_default()
}

fn read(database: &Path) -> rusqlite::Result<HashMap<String, i64>> {
    let connection = Connection::open_with_flags(
        database,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    let mut statement = connection.prepare(
        "SELECT targettedResource, MAX(lastUpdate) FROM ResourceScoreCache \
         WHERE targettedResource LIKE 'applications:%' GROUP BY targettedResource",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    Ok(rows
        .filter_map(Result::ok)
        .filter_map(|(resource, launched_at)| {
            let file_name = resource.strip_prefix(APPLICATION_PREFIX)?;
            let stem = file_name.strip_suffix(".desktop").unwrap_or(file_name);
            Some((stem.to_string(), launched_at))
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_last_launch_of_each_application() {
        let path = std::env::temp_dir().join(format!("konsol-kde-{}.sqlite", std::process::id()));
        let connection = Connection::open(&path).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE ResourceScoreCache (usedActivity TEXT, initiatingAgent TEXT, \
                 targettedResource TEXT, scoreType INT, cachedScore REAL, lastUpdate INT, firstUpdate INT);
                 INSERT INTO ResourceScoreCache VALUES ('a', 'x', 'applications:skate3recomp.desktop', 0, 1, 100, 50);
                 INSERT INTO ResourceScoreCache VALUES ('b', 'x', 'applications:skate3recomp.desktop', 0, 1, 300, 50);
                 INSERT INTO ResourceScoreCache VALUES ('a', 'x', 'file:///tmp/notes.txt', 0, 1, 900, 50);",
            )
            .unwrap();
        drop(connection);
        let launches = read(&path).unwrap();
        assert_eq!(launches.get("skate3recomp"), Some(&300));
        assert_eq!(launches.len(), 1);
        std::fs::remove_file(path).unwrap();
    }
}
