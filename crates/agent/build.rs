//! Build script for agent crate
//!
//! 负责打包内置技能 (Bundled Skills) 到二进制文件中

use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

fn main() -> io::Result<()> {
    // 打包内置技能
    bundle_skills()?;

    Ok(())
}

/// 打包内置技能目录到 tar.gz 格式
fn bundle_skills() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let bundled_skills_dir = PathBuf::from(&manifest_dir).join("bundled-skills");
    let output_file = PathBuf::from(&out_dir).join("bundled_skills.tar.gz");

    // 如果 bundled-skills 目录不存在，创建一个空的压缩文件
    if !bundled_skills_dir.exists() {
        fs::write(&output_file, [])?;
        println!("cargo:warning=Bundled skills directory not found, creating empty archive");
        return Ok(());
    }

    // 创建一个简单的 tar 归档（简化实现，不使用 tar crate）
    // 实际项目中应该使用 flate2 和 tar crate 来创建真正的压缩文件
    let mut archive_data = Vec::new();

    // 写入简单的头部信息
    writeln!(&mut archive_data, "BUNDLED_SKILLS_V1")?;

    // 遍历技能目录
    for entry in fs::read_dir(&bundled_skills_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let skill_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");

            // 查找 SKILL.md 文件
            let skill_md_path = path.join("SKILL.md");
            if skill_md_path.exists() {
                // 读取文件内容
                let content = fs::read_to_string(&skill_md_path)?;

                // 写入归档：技能名称 + 内容长度 + 内容
                writeln!(&mut archive_data, "SKILL:{}", skill_name)?;
                writeln!(&mut archive_data, "SIZE:{}", content.len())?;
                writeln!(&mut archive_data, "BEGIN_CONTENT")?;
                write!(&mut archive_data, "{}", content)?;
                writeln!(&mut archive_data, "\nEND_CONTENT")?;
            }
        }
    }

    // 写入输出文件
    fs::write(&output_file, &archive_data)?;

    // 告诉 Cargo 如果 bundled-skills 目录变化则重新运行
    println!("cargo:rerun-if-changed={}/bundled-skills", manifest_dir);

    Ok(())
}
