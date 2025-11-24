use anyhow::{Result, anyhow};
use keepass_ng::db::{Database, Group, NodePtr, Node, with_node};


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
    pub node: NodePtr,
}

impl<'a> Finder<'a> {
    pub fn new(db: &'a Database, options: SearchOptions) -> Self {
        Self { db, options }
    }

    pub fn find(&self, query: &str) -> Result<Vec<SearchResult>> {
        if query.starts_with('/') {
            self.find_by_absolute_path(query)
        } else if query.contains('/') {
            self.find_by_subpath(query)
        } else {
            self.find_by_name(query)
        }
    }

    fn find_by_absolute_path(&self, path: &str) -> Result<Vec<SearchResult>> {
        let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
        if parts.is_empty() {
            return Err(anyhow!("Empty path"));
        }

        // db.root is SerializableNodePtr which derefs to NodePtr
        // We need to treat it as a Group to get its name and children
        
        let root_ptr = &self.db.root;
        
        let root_name = with_node::<Group, _, _>(root_ptr, |g| g.get_title().unwrap_or("").to_string())
            .ok_or_else(|| anyhow!("Root is not a group"))?;
            
        // Check if path starts with root name
        // ... (logic remains similar but we need to work with NodePtr/Group)
        
        // Let's just start search from root.
        
        let start_index = if parts[0] == root_name { 1 } else { 0 };
        
        if start_index >= parts.len() {
             return Err(anyhow!("Cannot return root group as result"));
        }
        
        let mut current_node_ptr: NodePtr = (**root_ptr).clone(); 

        for i in start_index..parts.len() {
            let part = parts[i];
            let is_last_part = i == parts.len() - 1;
            
            // Access current group
            let (children, entries) = with_node::<Group, _, _>(&current_node_ptr, |g| {
                (g.groups(), g.entries())
            }).ok_or_else(|| anyhow!("Current node is not a group"))?;
            
            let mut found = false;
            
            if is_last_part {
                // Check entries first
                for entry in entries {
                    let title = entry.borrow().get_title().map(|s| s.to_string()).unwrap_or_default();
                    if title == part {
                        return Ok(vec![SearchResult {
                            path: path.to_string(),
                            node: entry,
                        }]);
                    }
                }
            }
            
            // Check subgroups
            for child in children {
                let title = child.borrow().get_title().map(|s| s.to_string()).unwrap_or_default();
                if title == part {
                    current_node_ptr = child;
                    found = true;
                    break;
                }
            }
            
            if !found && !is_last_part {
                return Err(anyhow!("Group not found: {}", part));
            }
            if !found && is_last_part {
                 // If we found a group (assigned to current_node_ptr), return it
                 return Ok(vec![SearchResult {
                     path: path.to_string(),
                     node: current_node_ptr,
                 }]);
            }
        }
        
        Err(anyhow!("Entry not found"))
    }

    fn find_by_subpath(&self, query: &str) -> Result<Vec<SearchResult>> {
        let parts: Vec<&str> = query.split('/').collect();
        if parts.len() < 2 {
            return Err(anyhow!("Invalid subpath query"));
        }

        let target_name = parts.last().unwrap();
        let sub_path = &parts[..parts.len() - 1];
        
        let mut results = Vec::new();
        // Start from root
        let root_ptr = &self.db.root;
        with_node::<Group, _, _>(root_ptr, |g| {
            self.search_group_recursive(g, "/", sub_path, target_name, &mut results)
        }).ok_or_else(|| anyhow!("Root is not a group"))??;
        
        // Filter results
        let filtered: Vec<SearchResult> = results.into_iter()
            .filter(|r| r.path.contains(query))
            .collect();

        Ok(filtered)
    }

    fn find_by_name(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let root_ptr = &self.db.root;
        with_node::<Group, _, _>(root_ptr, |g| {
            self.search_group_recursive_name(g, "", query, &mut results)
        }).ok_or_else(|| anyhow!("Root is not a group"))??;
        Ok(results)
    }

    fn search_group_recursive(
        &self,
        group: &Group,
        current_path: &str,
        search_path: &[&str],
        target_name: &str,
        results: &mut Vec<SearchResult>,
    ) -> Result<()> {
        let group_name = group.get_title().unwrap_or("");
        let group_path = if group_name.is_empty() {
            current_path.to_string()
        } else {
             if current_path == "/" {
                format!("/{}", group_name)
            } else {
                format!("{}/{}", current_path, group_name)
            }
        };

        // Check if we are at the target depth
        if search_path.len() == 1 {
             for entry in group.entries() {
                let title = entry.borrow().get_title().map(|s| s.to_string()).unwrap_or_default();
                if self.matches(&title, target_name) {
                     let full_path = format!("{}/{}", group_path, title);
                     results.push(SearchResult {
                        path: full_path,
                        node: entry,
                    });
                }
            }
        }

        if !search_path.is_empty() {
             if self.matches(group_name, search_path[0]) {
                 for child_ptr in group.groups() {
                     // We need to recursively call. But child_ptr is NodePtr.
                     // We need to borrow it as Group.
                     with_node::<Group, _, _>(&child_ptr, |child_group| {
                         let _ = self.search_group_recursive(child_group, &group_path, &search_path[1..], target_name, results);
                     });
                 }
             }
        }

        // Always search subgroups
        for child_ptr in group.groups() {
            with_node::<Group, _, _>(&child_ptr, |child_group| {
                 let _ = self.search_group_recursive(child_group, &group_path, search_path, target_name, results);
            });
        }

        Ok(())
    }

    fn search_group_recursive_name(
        &self,
        group: &Group,
        current_path: &str,
        target_name: &str,
        results: &mut Vec<SearchResult>,
    ) -> Result<()> {
         let group_name = group.get_title().unwrap_or("");
         
         let group_path = if group_name.is_empty() {
            current_path.to_string()
        } else {
            if current_path.is_empty() {
                group_name.to_string()
            } else {
                format!("{}/{}", current_path, group_name)
            }
        };

        for entry in group.entries() {
            let title = entry.borrow().get_title().map(|s| s.to_string()).unwrap_or_default();
            if self.matches(&title, target_name) {
                let full_path = format!("{}/{}", group_path, title);
                results.push(SearchResult {
                    path: format!("/{}", full_path),
                    node: entry,
                });
            }
        }

        for child_ptr in group.groups() {
            with_node::<Group, _, _>(&child_ptr, |child_group| {
                let _ = self.search_group_recursive_name(child_group, &group_path, target_name, results);
            });
        }

        Ok(())
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
