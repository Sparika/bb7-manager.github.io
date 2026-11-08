use anyhow::{Context, Result};
use serde_json;
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::path::{Path, PathBuf};
use unicode_normalization::UnicodeNormalization;

// Colonnes des joueurs dans le CSV (0-based)
const COL_NUM: usize = 0;
const COL_PLAYER: usize = 1;
const COL_POSITION: usize = 2;
const COL_MA: usize = 3;
const COL_ST: usize = 4;
const COL_AG: usize = 5;
const COL_PA: usize = 6;
const COL_AV: usize = 7;
const COL_SKILLS: usize = 8;
const COL_HIRING_FEE: usize = 9;
const COL_VALUE: usize = 19;

/// Structure pour stocker les informations d'un joueur
#[derive(Debug, Clone)]
struct PlayerInfo {
    num: String,
    name: String,
    position: String,
    ma: String,
    st: String,
    ag: String,
    pa: String,
    av: String,
    skills: String,
    hiring_fee: String,
    value: String,
}

/// Structure pour stocker les informations d'une équipe
#[derive(Debug, Clone)]
struct TeamInfo {
    race: String,
    french_name: String,
    file_name: String,
    cttv: Option<String>,
    treasury: Option<String>,
    rerolls: Option<String>,
    dedicated_fans: Option<String>,
    players: Vec<PlayerInfo>,
}

fn main() -> Result<()> {
    // Chemins relatifs au script (scripts/ -> ../teams/ et ../src/)
    let teams_dir = Path::new("../teams");
    let skills_json_path = Path::new("skills_all.json");
    
    // Charger toutes les compétences (français + anglais -> description)
    let skills_map = load_skills(skills_json_path)?;

    println!("📂 Lecture des fichiers CSV depuis {:?}", teams_dir);

    // Lister tous les fichiers CSV
    let csv_files = list_csv_files(teams_dir)?;
    
    if csv_files.is_empty() {
        println!("⚠️  Aucun fichier CSV trouvé dans {:?}", teams_dir);
        return Ok(());
    }

    println!("📄 Fichiers CSV trouvés : {:?}", csv_files);
    println!("📚 {} compétences chargées (FR+EN)", skills_map.len());

    // Lire les informations des équipes
    let teams_info = read_all_teams_info(&csv_files)?;

    // Générer le contenu markdown
    let _markdown_content = generate_markdown(&csv_files, &skills_map)?;
    
    // Générer les fichiers HTML dédiés pour impression
    generate_print_html_files(&teams_info, &skills_map)?;

    println!("✅ Génération terminée");
    Ok(())
}

/// Charge les compétences depuis le fichier JSON
fn load_skills(path: &Path) -> Result<std::collections::HashMap<String, String>> {
    use std::collections::HashMap;
    
    if !path.exists() {
        println!("⚠️  Fichier des compétences introuvable : {:?}", path);
        return Ok(HashMap::new());
    }
    
    let content = fs::read_to_string(path)
        .with_context(|| format!("Échec de la lecture de {:?}", path))?;
    
    let value: serde_json::Value = serde_json::from_str(&content)
        .with_context(|| format!("Échec du parsing JSON de {:?}", path))?;
    
    let mut skills = HashMap::new();
    if let serde_json::Value::Object(map) = value {
        for (name, desc) in map {
            if let Some(desc_str) = desc.as_str() {
                skills.insert(name, desc_str.to_string());
            }
        }
    }
    
    Ok(skills)
}

/// Liste tous les fichiers CSV dans un dossier
fn list_csv_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut csv_files = Vec::new();
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() && path.extension().map_or(false, |ext| ext == "csv") {
            csv_files.push(path);
        }
    }
    
    // Trier par nom de fichier pour un ordre déterministe
    csv_files.sort();
    Ok(csv_files)
}

/// Lit toutes les informations des équipes depuis les fichiers CSV
fn read_all_teams_info(csv_files: &[PathBuf]) -> Result<Vec<TeamInfo>> {
    let mut teams_info = Vec::new();
    
    for csv_path in csv_files.iter() {
        let team_info = read_team_info(csv_path)
            .with_context(|| format!("Échec de la lecture de {:?}", csv_path))?;
        teams_info.push(team_info);
    }
    
    Ok(teams_info)
}

/// Génère les fichiers HTML dédiés pour impression
fn generate_print_html_files(teams_info: &[TeamInfo], skills_map: &HashMap<String, String>) -> Result<()> {
    let html_dir = Path::new("../src/equipes/html");
    if !html_dir.exists() {
        fs::create_dir_all(html_dir)
            .with_context(|| format!("Impossible de créer {:?}", html_dir))?;
    }
    
    for team_info in teams_info {
        let team_skills = extract_team_skills(team_info, skills_map);
        let html_content = generate_print_html(team_info, &team_skills);
        
        let filename = format!("{}.html", slugify(&team_info.french_name));
        let filepath = html_dir.join(filename);
        
        fs::write(&filepath, &html_content)
            .with_context(|| format!("Échec de l'écriture dans {:?}", filepath))?;
        
        println!("✅ HTML imprimable : {:?}", filepath);
    }
    
    Ok(())
}

/// Génère le contenu markdown à partir des fichiers CSV
fn generate_markdown(csv_files: &[PathBuf], skills_map: &HashMap<String, String>) -> Result<String> {
    let mut content = String::new();
    
    // En-tête
    content.push_str("# Équipes\n\n");
    content.push_str("Liste des équipes participantes à la Ligue Blood Bowl 7s.\n\n");
    content.push_str("---\n\n");

    // Lire chaque fichier CSV
    let mut teams_info = Vec::new();

    for (index, csv_path) in csv_files.iter().enumerate() {
        let team_info = read_team_info(csv_path)
            .with_context(|| format!("Échec de la lecture de {:?}", csv_path))?;
        
        teams_info.push((index + 1, team_info));
    }

    // Créer un dossier pour les équipes si inexistant
    let teams_output_dir = Path::new("../src/equipes");
    if !teams_output_dir.exists() {
        fs::create_dir_all(teams_output_dir)
            .with_context(|| format!("Impossible de créer {:?}", teams_output_dir))?;
    }

    // Générer un fichier par équipe
    for (_num, team_info) in &teams_info {
        let filename = format!("{}.md", slugify(&team_info.french_name));
        let filepath = teams_output_dir.join(&filename);
        
        let mut team_content = String::new();
        
        // En-tête
        team_content.push_str(&format!("# {}\n\n", team_info.french_name));
        team_content.push_str(&format!("_Race : {}_\n\n", team_info.race));
        team_content.push_str("---\n\n");

        // Métadonnées
        team_content.push_str("## Informations générales\n\n");
        
        if let Some(cttv) = &team_info.cttv {
            team_content.push_str(&format!("- **CTTV** : {}\n", cttv));
        }
        if let Some(treasury) = &team_info.treasury {
            team_content.push_str(&format!("- **Trésorerie** : {}\n", treasury));
        }
        if let Some(rerolls) = &team_info.rerolls {
            team_content.push_str(&format!("- **Relances** : {}\n", rerolls));
        }
        if let Some(fans) = &team_info.dedicated_fans {
            team_content.push_str(&format!("- **Fans dévoués** : {}\n", fans));
        }
        team_content.push_str(&format!("- **Nombre de joueurs** : {}\n", team_info.players.len()));
        team_content.push_str(&format!("- **Fichier source** : `{}`\n\n", team_info.file_name));

        // Lien vers la version imprimable (entre métadonnées et liste des joueurs)
        let html_filename = format!("{}.html", slugify(&team_info.french_name));
        team_content.push_str(&format!("[Version imprimable (A4 Landscape)](html/{})  \n\n", html_filename));

        // Liste des joueurs
        if !team_info.players.is_empty() {
            team_content.push_str(&format!("## Liste des joueurs ({} joueurs)\n\n", team_info.players.len()));
            team_content.push_str("| N° | Position | Nom | MA | ST | AG | PA | AV | Compétences | Coût | Valeur |\n");
            team_content.push_str("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |\n");
            
            for player in &team_info.players {
                let name = if player.name.is_empty() {
                    String::new()
                } else {
                    player.name.clone()
                };
                team_content.push_str(&format!(
                    "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                    player.num,
                    player.position,
                    name,
                    player.ma,
                    player.st,
                    player.ag,
                    player.pa,
                    player.av,
                    player.skills,
                    player.hiring_fee,
                    player.value
                ));
            }
            team_content.push_str("\n");
        }
        
        // Règles des compétences de l'équipe
        let team_skills = extract_team_skills(team_info, skills_map);
        if !team_skills.is_empty() {
            team_content.push_str("## Règles des compétences\n\n");
            team_content.push_str(&format!("Cette équipe utilise {} compétence(s) unique(s) :\n\n", team_skills.len()));
            
            for (skill_name, skill_desc) in &team_skills {
                team_content.push_str(&format!("### {}\n\n", skill_name));
                team_content.push_str(&format!("{}\n\n", skill_desc));
                team_content.push_str("---\n\n");
            }
        }

        // Écrire le fichier de l'équipe
        fs::write(&filepath, &team_content)
            .with_context(|| format!("Échec de l'écriture dans {:?}", filepath))?;
        
        println!("✅ Équipe générée : {:?}", filepath);
    }

    // Générer la page index des équipes
    let mut index_content = String::new();
    index_content.push_str("# Équipes\n\n");
    index_content.push_str("Liste des équipes participantes à la Ligue Blood Bowl 7s.\n\n");
    index_content.push_str("---\n\n");

    // Tableau récapitulatif
    index_content.push_str("## Résumé\n\n");
    index_content.push_str("| N° | Race | Nom de l'équipe | Joueurs | CTTV |\n");
    index_content.push_str("| --- | --- | --- | --- | --- |\n");

    for (num, team_info) in &teams_info {
        let cttv = team_info.cttv.as_deref().unwrap_or("N/A");
        let filename = format!("{}.md", slugify(&team_info.french_name));
        index_content.push_str(&format!(
            "| {} | {} | [{}](equipes/{}) | {} | {} |\n", 
            num, team_info.race, team_info.french_name, filename, team_info.players.len(), cttv
        ));
    }

    index_content.push_str("\n");

    // Liste avec liens directs
    index_content.push_str("## Navigation rapide\n\n");
    for (num, team_info) in &teams_info {
        let filename = format!("{}.md", slugify(&team_info.french_name));
        index_content.push_str(&format!(
            "{} . [{}](equipes/{}) - {} joueurs, CTTV: {}\n",
            num, team_info.french_name, filename, team_info.players.len(), 
            team_info.cttv.as_deref().unwrap_or("N/A")
        ));
    }

    // Écrire la page index
    let index_path = Path::new("../src/equipes.md");
    fs::write(index_path, &index_content)
        .with_context(|| format!("Échec de l'écriture dans {:?}", index_path))?;
    
    println!("✅ Page index générée : {:?}", index_path);

    Ok(index_content)
}

/// Extrait les compétences uniques de l'équipe et les trie par ordre alphabétique
/// Utilise skills_map qui contient déjà les correspondances FR+EN -> description
fn extract_team_skills(
    team_info: &TeamInfo, 
    skills_map: &HashMap<String, String>
) -> Vec<(String, String)> {
    let mut skill_set = BTreeSet::new();
    
    // Collecter toutes les compétences des joueurs
    for player in &team_info.players {
        if !player.skills.is_empty() {
            // Retirer les guillemets du champ complet d'abord
            let mut skills_str = player.skills.clone();
            if skills_str.starts_with('"') && skills_str.ends_with('"') {
                skills_str = skills_str[1..skills_str.len()-1].to_string();
            }
            
            // Séparer les compétences par virgule
            for skill in skills_str.split(',') {
                let skill_name = skill.trim().to_string();
                skill_set.insert(skill_name);
            }
        }
    }
    
    // Convertir en Vec trié et récupérer les descriptions
    let mut skills_vec = Vec::new();
    for skill_name in &skill_set {
        // Chercher directement dans skills_map (contient FR+EN)
        let desc = skills_map.get(skill_name)
            .cloned()
            .or_else(|| {
                // Essayer avec la casse normalisée
                let skill_lower = skill_name.to_lowercase();
                skills_map.iter()
                    .find(|(k, _)| k.to_lowercase() == skill_lower)
                    .map(|(_, v)| v.clone())
            })
            .or_else(|| {
                // Essayer avec la version nettoyée (sans guillemets résiduels)
                let cleaned = skill_name.trim_matches('"').trim().to_string();
                skills_map.get(&cleaned).cloned()
            })
            .unwrap_or_else(|| {
                println!("⚠️ Description non trouvée pour la compétence : {}", skill_name);
                "Description non disponible".to_string()
            });
        
        skills_vec.push((skill_name.clone(), desc));
    }
    
    skills_vec
}



/// Convertit une chaîne en slug (pour les noms de fichiers)
fn slugify(name: &str) -> String {
    name.nfc().collect::<String>()
        .chars()
        .map(|c| match c {
            'à' | 'á' | 'â' | 'ã' | 'å' => 'a',
            'è' | 'é' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'ù' | 'ú' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            'ÿ' => 'y',
            'À' | 'Á' | 'Â' | 'Ã' | 'Å' => 'A',
            'È' | 'É' | 'Ê' | 'Ë' => 'E',
            'Î' | 'Ï' => 'I',
            'Ô' | 'Ö' => 'O',
            'Ù' | 'Ú' | 'Û' | 'Ü' => 'U',
            'Ç' => 'C',
            'Ñ' => 'N',
            'ñ' => 'n',
            _ if c.is_ascii_punctuation() && c != '-' => ' ',
            _ => c,
        })
        .collect::<String>()
        .to_lowercase()
        .replace(' ', "-")
        .replace("--", "-")
        .trim_matches('-')
        .to_string()
}

/// Mapping des noms de fichiers vers les noms d'équipes en français
fn get_french_team_name(file_stem: &str) -> String {
    match file_stem {
        "dark_elves" => "Elfes Noirs".to_string(),
        "elven_union" => "Union Elfique".to_string(),
        "humains" => "Humains".to_string(),
        "nains" => "Nains".to_string(),
        "nains_chaos" => "Nains du Chaos".to_string(),
        "tomb_kings" => "Roi des Tombes".to_string(),
        "undead" => "Morts-Vivants".to_string(),
        _ => file_stem.replace('_', " "),
    }
}

/// Lit les informations d'une équipe depuis un fichier CSV
fn read_team_info(csv_path: &Path) -> Result<TeamInfo> {
    let file = File::open(csv_path)
        .with_context(|| format!("Impossible d'ouvrir {:?}", csv_path))?;
    
    let reader = std::io::BufReader::new(file);
    let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

    if lines.is_empty() {
        let file_stem = csv_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let name = get_french_team_name(&file_stem);
        return Ok(TeamInfo {
            race: file_stem.clone(),
            french_name: name,
            file_name: csv_path.file_name().unwrap().to_string_lossy().to_string(),
            cttv: None,
            treasury: None,
            rerolls: None,
            dedicated_fans: None,
            players: Vec::new(),
        });
    }

    // Lire la première ligne qui contient Team Race et Team Name
    let first_line = &lines[0];
    let fields = parse_csv_line(first_line);
    
    let (race, french_name) = if fields.len() >= 4 {
        let race = fields[1].trim().to_string();
        let file_stem = csv_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let name = get_french_team_name(&file_stem);
        (race, name)
    } else {
        let file_stem = csv_path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let name = get_french_team_name(&file_stem);
        (file_stem.clone(), name)
    };

    // Trouver l'index où commencent les métadonnées (lignes avec 2 colonnes)
    let mut metadata_start = lines.len();
    for (i, line) in lines.iter().enumerate().rev() {
        let fields = parse_csv_line(line);
        if fields.len() == 2 && !fields[0].trim().is_empty() {
            // C'est une ligne de métadonnées
            metadata_start = i;
        } else if fields.len() > 2 {
            // On a trouvé une ligne avec plus de 2 colonnes
            // C'est la fin des données des joueurs
            break;
        }
    }

    // Parser les joueurs (lignes entre l'en-tête et les métadonnées)
    let mut players = Vec::new();
    // Commencer après la ligne d'en-tête (ligne 0) et la ligne des titres (ligne 1)
    for line in &lines[2..metadata_start] {
        let fields = parse_csv_line(line);
        if fields.len() >= 20 {
            let player = PlayerInfo {
                num: fields[COL_NUM].trim().to_string(),
                name: fields[COL_PLAYER].trim().to_string(),
                position: fields[COL_POSITION].trim().to_string(),
                ma: fields[COL_MA].trim().to_string(),
                st: fields[COL_ST].trim().to_string(),
                ag: fields[COL_AG].trim().to_string(),
                pa: fields[COL_PA].trim().to_string(),
                av: fields[COL_AV].trim().to_string(),
                skills: fields[COL_SKILLS].trim().to_string(),
                hiring_fee: fields[COL_HIRING_FEE].trim().to_string(),
                value: fields[COL_VALUE].trim().to_string(),
            };
            players.push(player);
        }
    }

    // Parser les métadonnées (dernières lignes)
    let mut metadata = HashMap::new();
    for line in &lines[metadata_start..] {
        let fields = parse_csv_line(line);
        if fields.len() >= 2 && !fields[0].trim().is_empty() {
            metadata.insert(fields[0].trim().to_string(), fields[1].trim().to_string());
        }
    }

    Ok(TeamInfo {
        race,
        french_name,
        file_name: csv_path.file_name().unwrap().to_string_lossy().to_string(),
        cttv: metadata.get("CTV").cloned().or_else(|| metadata.get("Team Value").cloned()),
        treasury: metadata.get("Treasury").cloned(),
        rerolls: metadata.get("Rerolls").cloned(),
        dedicated_fans: metadata.get("Dedicated fans").cloned(),
        players,
    })
}

/// Génère un fichier HTML dédié pour impression A4
fn generate_print_html(team_info: &TeamInfo, team_skills: &[(String, String)]) -> String {
    let team_name = &team_info.french_name;
    let race = &team_info.race;
    let cttv = team_info.cttv.as_deref().unwrap_or("NA");
    let treasury = team_info.treasury.as_deref().unwrap_or("NA");
    let rerolls = team_info.rerolls.as_deref().unwrap_or("NA");
    let fans = team_info.dedicated_fans.as_deref().unwrap_or("NA");
    let player_count = team_info.players.len();
    
    let mut html = String::new();
    
    // Début du HTML
    html.push_str(&format!(r#"<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <title>Feuille de match - {}</title>
    <style>"#, team_name));
    
    // CSS
    html.push_str(r#"
        @page {
            size: A4 landscape;
            margin: 10mm;
        }
        
        body {
            font-family: Arial, sans-serif;
            font-size: 8pt;
            line-height: 1.2;
            color: #000;
            margin: 0;
            padding: 0;
        }
        
        .header {
            text-align: center;
            margin: 0 0 5mm 0;
            page-break-after: avoid;
        }
        
        .header h1 {
            font-size: 16pt;
            margin: 0;
            padding: 0;
            font-weight: bold;
        }
        
        .header .race {
            font-size: 11pt;
            color: #666;
            font-style: italic;
            margin: 2mm 0 0 0;
        }
        
        .meta {
            font-size: 9pt;
            margin: 3mm 0 8mm 0;
            page-break-after: avoid;
        }
        
        .meta ul {
            list-style: none;
            padding: 0;
            margin: 0;
            display: flex;
            flex-wrap: wrap;
            gap: 8mm;
        }
        
        .meta li {
            padding: 1mm 0;
        }
        
        .players-table {
            width: 100%;
            border-collapse: collapse;
            margin: 5mm 0;
            font-size: 8pt;
        }
        
        .players-table th, .players-table td {
            border: 1px solid #ccc;
            padding: 2mm 1mm;
            text-align: center;
            vertical-align: middle;
        }
        
        .players-table th:nth-child(9), .players-table td:nth-child(9) {
            max-width: 100px;
            font-size: 7pt;
        }
        
        .players-table th {
            background-color: #f0f0f0;
            font-weight: bold;
        }
        
        .players-table tr:nth-child(even) {
            background-color: #f8f8f8;
        }
        
        .players-table tr {
            page-break-inside: avoid;
        }
        
        .skills-section {
            margin: 8mm 0 0 0;
        }
        
        .skills-section h2 {
            font-size: 12pt;
            border-bottom: 1px solid #666;
            padding: 2mm 0;
            margin: 0 0 5mm 0;
            page-break-after: avoid;
        }
        
        .skills-grid {
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 4mm;
        }
        
        .skill-card {
            border: 1px solid #ddd;
            padding: 3mm;
            background-color: #fff;
            page-break-inside: avoid;
        }
        
        .skill-card h3 {
            margin: 0 0 2mm 0;
            font-size: 9pt;
            border-bottom: 1px solid #eee;
            padding: 0 0 1mm 0;
        }
        
        .skill-card p {
            margin: 0;
            font-size: 8pt;
            line-height: 1.3;
        }
        
        @media screen {
            .print-button {
                display: inline-block;
                padding: 4px 10px;
                background: #4CAF50;
                color: white;
                text-decoration: none;
                border-radius: 3px;
                font-size: 9pt;
                margin: 5mm 0;
                cursor: pointer;
                border: none;
            }
        }
        
        @media print {
            .print-button { display: none !important; }
        }
        
        @page {
            @bottom-right {
                content: "Page " counter(page);
                font-size: 7pt;
                color: #666;
            }
        }
    </style>
</head>
<body>
    <div class="header">
        <h1>"#);
    html.push_str(team_name);
    html.push_str(r#"</h1>
        <p class="race">Race: "#);
    html.push_str(race);
    html.push_str(r#"</p>
    </div>
    
    <div class="meta">
        <ul>
            <li><strong>CTTV:</strong> "#);
    html.push_str(cttv);
    html.push_str(r#"</li>
            <li><strong>Treasury:</strong> "#);
    html.push_str(treasury);
    html.push_str(r#"</li>
            <li><strong>Rerolls:</strong> "#);
    html.push_str(rerolls);
    html.push_str(r#"</li>
            <li><strong>Fans:</strong> "#);
    html.push_str(fans);
    html.push_str(r#"</li>
            <li><strong>Players:</strong> "#);
    html.push_str(&player_count.to_string());
    html.push_str(r#"</li>
        </ul>
    </div>
    
    <h2>Team Sheet</h2>
    <table class="players-table">
        <thead>
            <tr>
                <th>N&#176;</th>
                <th>Position</th>
                <th>Name</th>
                <th>MA</th>
                <th>ST</th>
                <th>AG</th>
                <th>PA</th>
                <th>AV</th>
                <th>Skills</th>
                <th>Cost</th>
            </tr>
        </thead>
        <tbody>"#);
    
    // Tableau des joueurs
    for player in &team_info.players {
        let name = if player.name.is_empty() { "" } else { &player.name };
        html.push_str(&format!(
            r#"<tr>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
                <td>{}</td>
            </tr>"#,
            player.num, player.position, name, player.ma, player.st, 
            player.ag, player.pa, player.av, player.skills, player.hiring_fee
        ));
    }
    
    // Fin du tableau des joueurs
    html.push_str(r#"</tbody>
    </table>"#);
    
    // Compétences
    if !team_skills.is_empty() {
        html.push_str(r#"<div class="skills-section"><h2>Skills</h2>
    <div class="skills-grid">"#);
        
        for (skill_name, skill_desc) in team_skills {
            html.push_str(&format!(
                r#"<div class="skill-card">
                <h3>{}</h3>
                <p>{}</p>
            </div>"#,
                skill_name, skill_desc
            ));
        }
        
        html.push_str(r#"</div></div>"#);
    }
    
    // Bouton pour l'écran
    html.push_str(r#"
</body>
</html>"#);
    
    html
}

/// Parse une ligne CSV manuellement (sans la bibliothèque csv)
/// Gère les champs entre guillemets avec des virgules à l'intérieur
fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current_field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            c if c as u32 == 34 => {
                in_quotes = !in_quotes;
            }
            ',' => {
                if in_quotes {
                    current_field.push(c);
                } else {
                    fields.push(current_field);
                    current_field = String::new();
                }
            }
            _ => {
                current_field.push(c);
            }
        }
    }
    
    // Ajouter le dernier champ
    fields.push(current_field);
    
    fields
}
