# Atelier JDLL 2023-04-02: Clidle

Le but est de faire un idle game (ex: Cookie Clicker) en TUI, en Rust.

## Installer Rust
La méthode conseillée est d'installer rust avec rustup, qui permet notamment de gérer les version rust d'installées.
```
https://www.rust-lang.org/tools/install
```

> si vous suivez la procédure indiquée, n'oubliez pas d'exécuter la commande suggérer à la fin

L'outil qu'on va utiliser est cargo. C'est le getionnaire de paquet.

Cargo a besoin  qu'un projet rust ai une certaine structure.
Le ficher `Cargo.toml` décrit les dépendences, le type de binaire qu'on souhaite, et d'autres informations sur le projet. Pour plus d'info https://doc.rust-lang.org/cargo/reference/manifest.html

Le fichier `Cargo.lock` est généré à la compilation et décrit précisément toutes les dépendences récursives
avec la version exacte nécéssaire.

Le dossier `target/` contient le résultat de la compilation.

Le dossier `src/` contient le code. Ici on a un seul fichier `main.rs` qui est le point d'entrée. Pour plus d'infos 
sur le layout d'un projet rust https://doc.rust-lang.org/cargo/guide/project-layout.html

De plus ici le fichier items.json est utilisé par clidle.

## Jouer
`cargo build` pour compiler et `cargo run` pour exécuter.

A l'exécution, le programme prend le controle du terminal et affiche ce qu'on lui dit de desssiner
dans la fonction ui.

Pour jouer appuyer sur la touche `c` incrémente le nombre de ligne de code.
Appuyer sur `b` pour acheter des items, puis écrire le mot entre parenthèses, puis appuyer pour entrée,
pour effectivement acheter des items producteurs de code. Mais attention, il faut avoir 
suffisament de lignes de code.

Echap pour quitter le mode achat, et q pour quitter tout court.


## Pistes

- Gestion des erreurs plus uniformes (surtout ne pas paniquer) avec une crate comme thiserror ou anyhow
- Sauvegarder l'état du jeu dans un fichier (penser à utiliser serde)
- Augmenter le prix d'un item en fonction du nombre d'item de la même catégorie deja posséder
- Afficher le prix d'achat d'un item
- Implémenter la revente d'item
- Plus d'item différent
- bonus
    - Evenement temporaires qui demande d'interagir rapidement
    - Ameliorations achetable
- achievements
- remplacer les commandes par des boutons (voir les examples de la lib de tui https://github.com/fdehau/tui-rs, n'hésitez pas a la cloner et à exécuter localement les examples pour voir ce qui est possible)
- Barres de progession ? https://docs.rs/indicatif/latest/indicatif/
