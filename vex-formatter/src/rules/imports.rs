// Import sorting and formatting rules

/// Sort import statements alphabetically
pub fn sort_imports(imports: &mut [String]) {
    imports.sort();
}

/// Group imports by category (std, external, local)
pub fn group_imports(imports: &[String]) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut std_imports = Vec::new();
    let mut external_imports = Vec::new();
    let mut local_imports = Vec::new();

    for import in imports {
        if import.contains("\"std") {
            std_imports.push(import.clone());
        } else if import.starts_with("import") && import.contains("\"./") {
            local_imports.push(import.clone());
        } else {
            external_imports.push(import.clone());
        }
    }

    (std_imports, external_imports, local_imports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sort_imports() {
        let mut imports = vec![
            "import { z } from \"mod\";".to_string(),
            "import { a } from \"mod\";".to_string(),
            "import { m } from \"mod\";".to_string(),
        ];
        sort_imports(&mut imports);
        assert_eq!(imports[0], "import { a } from \"mod\";");
        assert_eq!(imports[1], "import { m } from \"mod\";");
        assert_eq!(imports[2], "import { z } from \"mod\";");
    }

    #[test]
    fn test_group_imports() {
        let imports = vec![
            "import { io } from \"std\";".to_string(),
            "import { http } from \"external\";".to_string(),
            "import { utils } from \"./utils\";".to_string(),
        ];

        let (std_imports, external_imports, local_imports) = group_imports(&imports);

        assert_eq!(std_imports.len(), 1);
        assert_eq!(external_imports.len(), 1);
        assert_eq!(local_imports.len(), 1);
    }
}
