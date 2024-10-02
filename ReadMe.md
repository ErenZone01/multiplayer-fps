# TODO pour Maze Wars

## 1. Configuration du projet
- [ ] Initialiser le projet Rust avec Bevy
- [ ] Configurer le fichier Cargo.toml avec les dépendances nécessaires
- [ ] Créer la structure de base du projet (dossiers et fichiers)

## 2. Serveur
- [ ] Implémenter la structure de base du serveur UDP
- [ ] Gérer les connexions entrantes des clients
- [ ] Implémenter un système pour maintenir l'état du jeu
- [ ] Créer un mécanisme pour diffuser les mises à jour aux clients
- [ ] Gérer la déconnexion des clients
- [ ] Optimiser le serveur pour gérer au moins 10 connexions simultanées

## 3. Client
- [ ] Créer l'interface de connexion (saisie de l'IP du serveur et du nom d'utilisateur)
- [ ] Implémenter la connexion UDP au serveur
- [ ] Gérer l'envoi et la réception des messages avec le serveur
- [ ] Implémenter la boucle principale du jeu

## 4. Génération du labyrinthe
- [ ] Concevoir une structure de données pour représenter le labyrinthe
- [ ] Implémenter un algorithme de génération de labyrinthe
- [ ] Créer au moins 3 niveaux de difficulté croissante
- [ ] Assurer que les labyrinthes générés sont solvables

## 5. Interface utilisateur
- [ ] Créer la fenêtre principale du jeu avec ggez
- [ ] Implémenter le rendu du labyrinthe en 3D (style Maze Wars original)
- [ ] Ajouter une mini-carte montrant la position du joueur et le labyrinthe complet
- [ ] Afficher le taux de rafraîchissement (FPS) à l'écran
- [ ] Créer une interface pour la sélection des niveaux

## 6. Logique du jeu
- [ ] Implémenter le mouvement du joueur
- [ ] Gérer les collisions avec les murs du labyrinthe
- [ ] Ajouter la logique pour les tirs et les hits
- [ ] Implémenter un système de score
- [ ] Gérer la synchronisation de l'état du jeu entre le serveur et les clients

## 7. Multijoueur
- [ ] Afficher les autres joueurs dans le labyrinthe
- [ ] Synchroniser les positions et actions des joueurs via le réseau
- [ ] Implémenter un système de spawn des joueurs

## 8. Optimisation des performances
- [ ] Optimiser le rendu pour maintenir un taux de rafraîchissement supérieur à 50 FPS
- [ ] Optimiser la communication réseau pour réduire la latence
- [ ] Profiler et optimiser l'utilisation de la mémoire

## 9. Tests et débogage
- [ ] Écrire des tests unitaires pour les composants critiques
- [ ] Effectuer des tests d'intégration pour le client et le serveur
- [ ] Tester le jeu avec plusieurs clients connectés simultanément
- [ ] Déboguer et résoudre les problèmes de synchronisation ou de gameplay

## 10. Fonctionnalités bonus (optionnelles)
- [ ] Implémenter un éditeur de niveau
- [ ] Créer un algorithme de génération automatique de nouveaux labyrinthes
- [ ] Ajouter des joueurs IA utilisant des algorithmes de pathfinding
- [ ] Améliorer l'interface de connexion avec une interface graphique et un historique des serveurs

## 11. Documentation et finalisation
- [ ] Écrire une documentation détaillée du code
- [ ] Créer un README expliquant comment installer et lancer le jeu
- [ ] Préparer une démonstration du jeu
- [ ] Effectuer une revue finale du code et des fonctionnalités
