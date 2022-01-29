const TEST:bool = false;
#[test]
fn test() {}


fn main() {
    #[cfg(all(feature = "bass_audio", feature = "neb_audio"))] 
    panic!("\n\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\nfeatures `bass_audio` and `neb_audio` cannot be used at the same time!\nTo use neb audio, disable default features\n!!!!!!!!!!!!!!!!!!!!!!!!!!!\n\n");

    // write commits file
    // read env vars from CI
    macro_rules! env {
        ($key:expr, $tst_str:expr) => {
            if TEST {
                $tst_str.to_owned()
            } else {
                match std::env::var($key) {
                    Ok(val) => val,
                    Err(_) => return,
                }
            }
        }
    }
    macro_rules! ok {
        ($thing:expr) => {
            match $thing {
                Ok(thing) => thing,
                // if theres an error while testing, panic
                Err(e) if TEST => panic!("err: {}", e),
                // otherwise exit the fn
                Err(_) => return
            }
        };
    }

    if !TEST {
        if let Err(_) = std::env::var("CI_PROJECT_ID") {
            let dir = std::env::current_dir().unwrap().to_string_lossy().to_string();
            let commit_file = format!(
                "{}/{}/src/{}commits.rs", 
                dir,
                if dir.ends_with("tataku-client") {"."} else {"tataku-client"},
                ""
                // if TEST {"test-"} else {""}
            );
            std::fs::write(
                commit_file, 
                build_commits_file(Vec::new(), String::new())
            ).expect("error writing commits.rs");
            return
        }
    }

    //TODO: try using git to get some of these
    let id = env!("CI_PROJECT_ID", "77");
    let dir = env!("CI_PROJECT_DIR", std::env::current_dir().unwrap().to_string_lossy().to_string());
    let branch = env!("CI_COMMIT_BRANCH", "main");
    let url = env!("CI_API_V4_URL", "https://gitlab.ayyeve.xyz/api/v4");
    let this_commit = env!("CI_COMMIT_SHA", "1bc485e2bc088d837d893cdd22a04dc92dccd95d");

    let commit_file = format!(
        "{}/{}/src/{}commits.rs", 
        dir,
        if dir.ends_with("tataku-client") {"."} else {"tataku-client"},
        ""
        // if TEST {"test-"} else {""}
    );
    println!("dir: {:?}, path: {}", dir, commit_file);

    // build the query url
    let url = format!("{}/projects/{}/repository/commits?ref_name={}", url, id, branch);

    // perform the query
    let res = ok!(reqwest::blocking::get(url));
    
    // get the data
    let response_data = ok!(res.text());
    
    // convert the data
    let commits:Vec<GitCommit> = ok!(serde_json::from_str(&response_data));

    // write the commits file
    std::fs::write(
        commit_file, 
        build_commits_file(commits, this_commit)
    ).expect("error writing commits.rs");

    // if TEST {panic!("end of test")};
}


fn build_commits_file(commits: Vec<GitCommit>, commit_hash: String) -> String {
    const TUPLE_TYPE:&'static str = "&'static str";
    const TUPLE_COUNT:usize = 5;

    let mut output = String::new();
    output += &format!("pub const COMMIT_HASH: &'static str = \"{}\";\n", commit_hash);
    output += &format!("pub const COMMITS:&[({})] = &[\n", [TUPLE_TYPE; TUPLE_COUNT].join(","));

    for commit in commits.iter() {
        output += &format!(
            "    (\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"),\n",
            clean_string(&commit.id),
            clean_string(&commit.title),
            clean_string(&commit.message),
            clean_string(&commit.committed_date),
            clean_string(&commit.web_url)
        )
    }

    output += "];";
    output
}


fn clean_string(s:&String) -> String {
    s
    .replace("\"", "\\\"")
    
    .trim()
    .to_owned()
}

#[derive(serde::Deserialize)]
pub struct GitCommit {
    /// hash
    pub id: String,
    /// short hash
    pub short_id: String,
    pub title: String,
    pub author_name: String,
    pub author_email: String,
    pub authored_date: String,
    pub committer_name: String,
    pub committer_email: String,
    pub committed_date: String,
    pub created_at: String,
    pub message: String,
    pub parent_ids: Vec<String>,
    pub web_url: String
}
