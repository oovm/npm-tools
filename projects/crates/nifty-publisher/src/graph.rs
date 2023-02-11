use std::collections::{BTreeMap, HashMap};

use petgraph::algo::{is_cyclic_directed, tarjan_scc, toposort};
use petgraph::{Directed, Graph};

use crate::workspace::{NpmPackage, collect_internal_dependency_names};
use crate::Result;

/// Topological publish order: dependencies before dependents.
pub fn sort_packages_for_publish(packages: &[NpmPackage], by_name: &BTreeMap<String, NpmPackage>) -> Result<Vec<String>> {
    let mut graph: Graph<String, (), Directed> = Graph::new();
    let mut node_indices = HashMap::new();

    for package in packages {
        let index = graph.add_node(package.name.clone());
        node_indices.insert(package.name.clone(), index);
    }

    for package in packages {
        let to_index = node_indices[&package.name];
        for dep in collect_internal_dependency_names(package, by_name) {
            if let Some(from_index) = node_indices.get(&dep) {
                graph.add_edge(*from_index, to_index, ());
            }
        }
    }

    match toposort(&graph, None) {
        Ok(sorted_indices) => Ok(sorted_indices.into_iter().map(|index| graph[index].clone()).collect()),
        Err(_) => {
            if is_cyclic_directed(&graph) {
                let cycles = tarjan_scc(&graph)
                    .into_iter()
                    .filter(|component| component.len() > 1)
                    .map(|component| {
                        component
                            .into_iter()
                            .map(|index| graph[index].clone())
                            .collect::<Vec<_>>()
                            .join(" -> ")
                    })
                    .collect::<Vec<_>>();
                Err(format!("cyclic workspace dependency among npm packages: {}", cycles.join(", ")))
            } else {
                Err("topological sort failed".into())
            }
        }
    }
}

/// Names of publishable (non-private) packages in dependency order.
pub fn plan_publish_order(all_packages: &[NpmPackage]) -> Result<Vec<String>> {
    let by_name = all_packages
        .iter()
        .map(|package| (package.name.clone(), package.clone()))
        .collect::<BTreeMap<_, _>>();
    let publishable: Vec<NpmPackage> = all_packages.iter().filter(|package| !package.private).cloned().collect();
    sort_packages_for_publish(&publishable, &by_name)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    use super::plan_publish_order;
    use crate::workspace::{NpmPackage, PackageManifest};

    fn package(name: &str, deps: &[(&str, &str)]) -> NpmPackage {
        let mut dependencies = BTreeMap::new();
        for (dep_name, spec) in deps {
            dependencies.insert((*dep_name).to_string(), (*spec).to_string());
        }
        let manifest = PackageManifest {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            private: false,
            dependencies,
            dev_dependencies: BTreeMap::new(),
            optional_dependencies: BTreeMap::new(),
            peer_dependencies: BTreeMap::new(),
        };
        NpmPackage {
            name: name.to_string(),
            version: "0.0.0".to_string(),
            dir: PathBuf::from(name),
            manifest_path: PathBuf::from(name).join("package.json"),
            private: false,
            manifest,
        }
    }

    #[test]
    fn orders_dependencies_before_dependents() {
        let packages = vec![
            package("@scope/main", &[("@scope/platform", "0.0.0")]),
            package("@scope/platform", &[]),
        ];
        let order = plan_publish_order(&packages).expect("order");
        assert_eq!(order, vec!["@scope/platform".to_string(), "@scope/main".to_string()]);
    }

    #[test]
    fn skips_private_packages_in_plan() {
        let mut private = package("@scope/private", &[]);
        private.private = true;
        private.manifest.private = true;
        let packages = vec![package("@scope/main", &[]), private];
        let order = plan_publish_order(&packages).expect("order");
        assert_eq!(order, vec!["@scope/main".to_string()]);
    }
}
