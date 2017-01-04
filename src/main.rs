#[macro_use] extern crate lazy_static;
extern crate git2;
extern crate regex;

use std::env;
use std::path::PathBuf;

use git2::Repository;
use regex::Regex;

fn get_latest_commit(repo: &Repository) -> git2::Oid {
    let master_branch = match repo.find_branch("master", git2::BranchType::Local) {
        Ok(branch) => branch,
        Err(e) => panic!("failed to find branch 'master': {}", e),
    };
    if !master_branch.is_head() {
        panic!("master is not HEAD. you're ded. D-E-D.");
    }
    let branch_ref = master_branch.get();
    let oid = match branch_ref.target() {
        Some(target) => target,
        None => panic!("failed to get oid for master branch ref"),
    };
    oid
}

fn print_url(repo: &Repository, remote_name: &str) {
    // Pattern to match against remote URL's. Ensure it's compiled only once.
    lazy_static! {
        static ref URL_REGEX: Regex = Regex::new(r"^(https://|git@)github\.com(/|:)(?P<org>.*?)/(?P<repo>.*?)\.git$").unwrap();
    }

    // Get latest commit
    let oid = get_latest_commit(repo);

    let target_remote = match repo.find_remote(remote_name) {
        Ok(remote) => remote,
        Err(e) => panic!("no remote found with name of '{}', {}", remote_name, e),
    };
    let remote_url = match target_remote.url() {
        Some(url) => url,
        None => panic!("couldn't get url for remote '{}'", target_remote.name().unwrap_or("<unnamed>")),
    };
    // println!("Remote URL is '{}'", remote_url);
    if URL_REGEX.is_match(remote_url) {
        let captures = URL_REGEX.captures(remote_url).unwrap();
        let org = match captures.name("org") {
            Some(org) => org.as_str(),
            None => panic!("could not match organization/user in remote URL for Github repo at '{}'", remote_url),
        };
        let repo_name = match captures.name("repo") {
            Some(repo_name) => repo_name.as_str(),
            None => panic!("could not match repository name in remote URL for Github repo at '{}'", remote_url),
        };
        // Point of this whole program:
        println!("https://github.com/{}/{}/blob/{}/project.clj#L1-L2", org, repo_name, oid);
        // TODO Enhance this by accepting relative file name and line ranges as CLI args
    } else {
        panic!("remote url '{}' for remote '{}' does not match expected URL pattern", remote_url, target_remote.name().unwrap());
    }
}

fn main() {
    let working_dir = if env::args().len() > 1 {
        let string_path = match env::args().nth(1) {
            Some(path) => path,
            None => panic!("rust itself is broken")
        };
        // println!("Using {}", string_path);
        PathBuf::from(string_path)
    } else {
        panic!("You must supply the root directory of the target Git repository.");
    };
    let repo = match Repository::open(working_dir) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to open repo: {}", e),
    };
    let remotes = match repo.remotes() {
        Ok(strings) => strings,
        Err(e) => panic!("could not get remotes for repo, {}", e),
    };
    if remotes.len() == 0 {
        panic!("This repository has no remotes, so making a link for it makes no sense.");
    }
    if remotes.len() == 1 {
        let remote_name = remotes.get(0).unwrap();
        print_url(&repo, remote_name);
    } else {
        if env::args().len() > 2 {
            let remote_name = match env::args().nth(2) {
                Some(name) => name,
                None => panic!("rust itself is broken")
            };
            print_url(&repo, &remote_name);
        } else {
            panic!("This repo has multiple remotes, so you need to specify which one to use.")
        }
    }
}
