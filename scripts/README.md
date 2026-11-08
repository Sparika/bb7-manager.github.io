# Script de génération des équipes

Ce script Rust génère automatiquement le fichier `src/equipes.md` à partir des fichiers CSV situés dans le dossier `teams/`.

## Structure

```
scripts/
├── Cargo.toml    # Configuration Cargo
├── README.md     # Ce fichier
└── src/
    └── main.rs   # Code source du script
```

## Prérequis

- Rust (via [rustup](https://rustup.rs/))
- Cargo (inclus avec Rust)

## Installation

```bash
# Se placer dans le dossier scripts
cd scripts

# Compiler le script (développement)
cargo build

# Compiler le script (release)
cargo build --release
```

## Utilisation

### Générer manuellement

```bash
# Depuis le dossier scripts
cargo run

# Ou depuis la racine du projet
./scripts/target/debug/generate_equipes
./scripts/target/release/generate_equipes  # version optimisée
```

### Utilisation avec Makefile

Le Makefile à la racine du projet propose plusieurs commandes :

```bash
# Générer uniquement src/equipes.md
make generate-equipes

# Builder le livre complet (génère équipes.md puis build)
make build

# Nettoyer
make clean

# Lancer le serveur de prévisualisation
make serve

# Builder et ouvrir dans le navigateur
make view
```

## Fonctionnalités

Le script extrait les informations suivantes des fichiers CSV :

- **Race** : Nom de la race en anglais (ex: Dark Elves, Chaos Dwarves)
- **Nom de l'équipe** : Traduction en français (ex: Elfes Noirs, Nains du Chaos)
- **CTTV** : Coach Team Value (valeur totale de l'équipe)
- **Trésorerie** : Argent disponible
- **Relances** : Nombre de relances
- **Fans dévoués** : Nombre de fans dévoués
- **Joueurs** : Liste complète des joueurs avec leurs caractéristiques et compétences

### Informations des joueurs

Pour chaque joueur, le script extrait :
- **N°** : Numéro du joueur
- **Position** : Poste du joueur
- **Nom** : Nom personnalisé (si présent)
- **MA** : Mouvement
- **ST** : Force
- **AG** : Agilité
- **PA** : Passe
- **AV** : Armure
- **Compétences** : Liste des compétences et traits
- **Coût** : Coût d'embauche
- **Valeur** : Valeur actuelle

### Mapping des noms français

| Fichier CSV | Nom français |
|-------------|--------------|
| dark_elves.csv | Elfes Noirs |
| elven_union.csv | Union Elfique |
| humains.csv | Humains |
| nains.csv | Nains |
| nains_chaos.csv | Nains du Chaos |
| tomb_kings.csv | Roi des Tombes |
| undead.csv | Morts-Vivants |

Pour ajouter de nouvelles équipes, modifier la fonction `get_french_team_name` dans `src/main.rs`.

## Sortie générée

Le script génère maintenant **une page par équipe** :

### Structure générée
```
src/
├── equipes.md                    # Page index avec tableau récapitulatif
└── equipes/
    ├── elfes-noirs.md            # Page dédiée aux Elfes Noirs
    ├── union-elfique.md          # Page dédiée à l'Union Elfique
    ├── humains.md                # Page dédiée aux Humains
    ├── nains.md                  # Page dédiée aux Nains
    ├── nains-du-chaos.md         # Page dédiée aux Nains du Chaos
    ├── roi-des-tombes.md         # Page dédiée au Roi des Tombes
    └── morts-vivants.md          # Page dédiée aux Morts-Vivants
```

### Contenu de chaque page d'équipe

**Page index (`src/equipes.md`)** :
- Tableau récapitulatif avec liens vers chaque équipe
- Liste de navigation rapide

**Page par équipe (ex: `src/equipes/elfes-noirs.md`)** :
```markdown
# Elfes Noirs

_Race : Dark Elves_

---

## Informations générales

- **CTTV** : 595000
- **Trésorerie** : 5000
- **Relances** : 0
- **Fans dévoués** : 1
- **Nombre de joueurs** : 7
- **Fichier source** : `dark_elves.csv`

## Liste des joueurs (7 joueurs)

| N° | Position | Nom | MA | ST | AG | PA | AV | Compétences | Coût | Valeur |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | Lineman |  | 6 | 3 | 2 | 3 | 9 |  | 65 000 | 65 000 |
| 4 | Runner |  | 7 | 3 | 2 | 3 | 8 | Délestage, Dégagement | 80 000 | 80 000 |
```

Le SUMMARY.md est automatiquement mis à jour pour inclure les liens vers toutes les pages d'équipes.

## Structure des fichiers CSV

Les fichiers CSV dans `teams/` ont la structure suivante :

```csv
"Team Race","Dark Elves","Team Name","My Team"
"#","Player","Position","MA","ST",...
"1","","Lineman","6","3",...
...
"Treasury","5000"
"Dedicated fans","1"
"Rerolls","0"
"CTV","595000"
"Team Value","595000"
```

Le script parse :
- La première ligne pour extraire la race
- Les dernières lignes pour extraire les métadonnées (CTV, Trésorerie, etc.)

## Personnalisation

### Ajouter une nouvelle équipe

1. Créer un fichier `teams/nom_equipe.csv`
2. Ajouter le mapping dans `get_french_team_name()` :

```rust
fn get_french_team_name(file_stem: &str) -> String {
    match file_stem {
        "nom_equipe" => "Nom en Français".to_string(),
        // ... autres mappings
        _ => file_stem.replace('_', " "),
    }
}
```

### Extraire d'autres métadonnées

Modifier la fonction `read_team_info()` pour extraire d'autres champs des métadonnées.

## Dépannage

### Erreur : "Impossible d'ouvrir ../teams/..."

Vérifiez que le script est exécuté depuis le dossier `scripts/`.

### Erreur : "Aucun fichier CSV trouvé"

Vérifiez que le dossier `teams/` existe et contient des fichiers `.csv`.

### Le fichier généré ne contient pas les bonnes informations

Vérifiez la structure de vos fichiers CSV. La première ligne doit contenir la race, et les métadonnées doivent être à la fin avec le format `"Clé","Valeur"`.
