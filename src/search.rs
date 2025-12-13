use anyhow::{anyhow, Result};
use keepass::db::{Entry, Group, Node};
use keepass::Database;

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub exact_match: bool,
}

pub struct Finder<'a> {
    db: &'a Database,
    options: SearchOptions,
}

#[derive(Debug)]
pub struct SearchResult {
    pub path: String,
    pub entry: Entry, // Return cloned Entry
}

impl<'a> Finder<'a> {
    pub fn new(db: &'a Database, options: SearchOptions) -> Self {
        Self { db, options }
    }

    pub fn find(&self, query: &str) -> Result<Vec<SearchResult>> {
        if query.starts_with('/') {
            return self.find_by_absolute_path(query);
        }
        self.find_by_search(query)
    }

    fn find_by_absolute_path(&self, path: &str) -> Result<Vec<SearchResult>> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty path"));
        }

        let root = &self.db.root;

        // If path starts with root name, skip it
        let start_index = if parts[0] == root.name { 1 } else { 0 };

        if start_index >= parts.len() {
            return Err(anyhow!("Cannot return root group as result"));
        }

        let mut current_group = root;

        for i in start_index..parts.len() {
            let part = parts[i];
            let is_last_part = i == parts.len() - 1;

            let mut found_group = None;

            // Check subgroups
            for child in &current_group.children {
                if let Node::Group(g) = child {
                    if g.name == part {
                        found_group = Some(g);
                        break;
                    }
                }
            }

            if let Some(g) = found_group {
                if is_last_part {
                    // Path points to a group. We usually want entries.
                    // But maybe we should return all entries in this group?
                    // Or maybe the user meant an entry with this name?
                    // Let's check if there is an entry with this name in current_group first.
                }
                current_group = g;
            } else if is_last_part {
                // Check entries in current_group
                for entry in current_group.entries() {
                    let title = entry.get_title().unwrap_or("");
                    if title == part {
                        return Ok(vec![SearchResult {
                            path: path.to_string(),
                            entry: entry.clone(),
                        }]);
                    }
                }
                return Err(anyhow!("Entry not found: {}", part));
            } else {
                return Err(anyhow!("Group not found: {}", part));
            }
        }

        // If we ended up at a group, maybe return all entries?
        // For now, let's say we only support finding specific entries.
        Err(anyhow!("Path points to a group, not an entry"))
    }

    fn find_by_search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let root = &self.db.root;
        // Start recursion. Root name is usually not part of the path in results if we want /Group/Entry
        // But if root has a name, it might be.
        // Let's assume root name is not part of the path for now, or handle it inside.
        // If we pass empty string as current_path, the logic inside handles it.
        self.search_recursive(root, "", query, &mut results);
        Ok(results)
    }

    fn search_recursive(
        &self,
        group: &Group,
        current_path: &str,
        query: &str,
        results: &mut Vec<SearchResult>,
    ) {
        let group_name = &group.name;

        let group_path = if current_path.is_empty() {
            if group_name == "Root" || group_name.is_empty() {
                String::new()
            } else {
                group_name.to_string()
            }
        } else {
            format!("{}/{}", current_path, group_name)
        };

        for entry in group.entries() {
            let title = entry.get_title().unwrap_or("");
            if self.matches(title, query) {
                let full_path = if group_path.is_empty() {
                    title.to_string()
                } else {
                    format!("{}/{}", group_path, title)
                };

                results.push(SearchResult {
                    path: format!("/{}", full_path),
                    entry: entry.clone(),
                });
            }
        }

        for child_group in group.groups() {
            self.search_recursive(child_group, &group_path, query, results);
        }
    }

    fn matches(&self, value: &str, pattern: &str) -> bool {
        if self.options.case_sensitive {
            if self.options.exact_match {
                value == pattern
            } else {
                value.contains(pattern)
            }
        } else {
            let value_lower = value.to_lowercase();
            let pattern_lower = pattern.to_lowercase();
            if self.options.exact_match {
                value_lower == pattern_lower
            } else {
                value_lower.contains(&pattern_lower)
            }
        }
    }
}
