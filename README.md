# Ligue Blood Bowl 7s

Projet MdBook de gestion d'une ligue de Blood Bowl Sevens en entreprise.

## Format

- Phase régulière : toutes les équipes se rencontrent une fois (aller uniquement), 1 match par équipe et par période de 3 semaines.
- Période 1 : du lundi 9 novembre au dimanche 29 novembre 2026.
- Congés banalisés sur les deux dernières semaines de décembre : la période 3 démarre le lundi 4 janvier 2027.
- Nombre d'équipes non figé : variantes prévues pour 6, 7 et 8 équipes (7 équipes inscrites à ce jour, plus 2 inscriptions potentielles).
- Un tirage au sort avant la période 1 affecte chaque coach à un numéro d'équipe, définitif pour toute la saison.
- Play-offs à élimination directe après la phase régulière : demi-finales + finale (6 et 7 équipes), quarts + demi-finales + finale (8 équipes).
- Barème : victoire 3 points, nul 2 points, défaite 1 point, forfait 0 point.
- Un match non joué dans sa période est rejoué en priorité la période suivante ; à défaut, forfait des deux équipes.

## Équipes

7 équipes inscrites : Nains du Chaos, Roi des Tombes, Morts-Vivants, Union Elfique, Elfes Noirs, Humains, Nains. Inscriptions potentielles : Bretonniens, Skavens.

## Structure

```text
├── book.toml                  Configuration MdBook (titre, langue fr, thème)
├── README.md                  Ce fichier
└── src/
    ├── SUMMARY.md             Sommaire
    ├── preambule.md           Présentation du livre
    ├── format-et-reglement.md Format, play-offs, barème
    ├── planning.md            Plannings des 3 variantes avec dates
    ├── gestion-des-rencontres.md  Feuille de tirage, grille de suivi, feuille de match, classement
    ├── tirage-au-sort.md      Mode d'emploi de la fiche de tirage
    ├── tirage-au-sort.html    Fiche A4 à découper pour le tirage (9 bons)
    └── feuille-de-match.html  Feuille de match A4 imprimable (1 par rencontre)
```

## Construire et publier

```bash
mdbook build        # génère le livre dans book/
mdbook serve        # prévisualisation locale sur http://localhost:3000
```

Les fichiers HTML de src sont copiés tels quels dans le livre généré : la fiche de tirage est accessible à `tirage-au-sort.html` et la feuille de match à `feuille-de-match.html` à la racine du livre.

## Suivi du projet

Les contenus de ce livre sont édités avec Vibe (Mistral AI), en session Work avec accès terminal. Chaque fichier correspond à un canvas Vibe ; pour modifier un chapitre, demander la mise à jour du canvas correspondant puis recopier le contenu ici. État au 7 novembre 2026 : plannings des 3 variantes établis, fiche de tirage et feuille de match prêtes, barème à jour (nul 2 points, défaite 1 point, forfait 0 point). Prochaine étape : tirage au sort avant le lundi 9 novembre 2026, puis suppression des variantes non retenues après confirmation du nombre d'équipes.
