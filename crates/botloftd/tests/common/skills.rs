//! The owner's skills folder the skill tests start from (spec 38).

use std::path::Path;

/// The owner's skills folder: two of their own and one from claude.ai.
pub fn owner_skills(root: &Path) {
    let deploy = root.join("deploy");
    std::fs::create_dir_all(deploy.join("scripts")).expect("deploy");
    std::fs::write(
        deploy.join("SKILL.md"),
        "---\nname: deploy\ndescription: Ships the app.\nallowed-tools: Bash(npm run build *)\n---\nRun the build.\n",
    )
    .expect("skill");
    std::fs::write(deploy.join("scripts").join("ship.py"), "print('ship')").expect("script");
    let notes = root.join("notes");
    std::fs::create_dir_all(&notes).expect("notes");
    std::fs::write(
        notes.join("SKILL.md"),
        "---\ndescription: Keeps notes.\n---\nNotes.\n",
    )
    .expect("skill");
    let pdf = root.join("synced").join("account-1").join("pdf");
    std::fs::create_dir_all(&pdf).expect("pdf");
    std::fs::write(
        pdf.join("SKILL.md"),
        "---\ndescription: Reads PDFs.\n---\nPDFs.\n",
    )
    .expect("skill");
}
