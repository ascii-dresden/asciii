// This file was generated automatically by crowbook-localize.
// It is probably not a good idea to edit it manually.
//
// # Usage:
//
// ```rust, no_run
// extern crate crowbook_intl_runtime;
// #[macro_use] mod localize_macros;
// crowbook_intl_runtime::set_lang("en");
// lformat!("Hello, {}", name);
// set_lang("fr");
// lformat!("Hello, {}", name);
// ```


/// Localized format macro (or `lformat!` in short)
///
/// Should be similar to `format!`, except strings are localized.
/// Generated automatically, you should not edit it.
macro_rules! lformat {
    ("Pay {}\nYou have had the money for {} days!", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Bitte bezahle {}, die Rechnung ist seit {} Tagen bezahlt", $($arg)*),
            _ => format!("Pay {}\nYou have had the money for {} days!", $($arg)*),
        }
    });
    ("Inquire about: \"{event}\"!", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Erkundige dich über \"{event}\"!", $($arg)*),
            _ => format!("Inquire about: \"{event}\"!", $($arg)*),
        }
    });
    ("{:?} has been finished for {} days, get rid of it!", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("{:?} ist seit {} Tagen abgeschlossen. Weg damit!", $($arg)*),
            _ => format!("{:?} has been finished for {} days, get rid of it!", $($arg)*),
        }
    });
    ("{}: Hungry employees!", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("{}: Hungrige Mitarbeiter!", $($arg)*),
            _ => format!("{}: Hungry employees!", $($arg)*),
        }
    });
    ("{} weeks", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("{} Wochen", $($arg)*),
            _ => format!("{} weeks", $($arg)*),
        }
    });
    ("{rnum}: payment is {weeks} weeks late: \"{event}\"", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("{rnum}: Zahlungsverzug {weeks} Wochen: \"{event}\"", $($arg)*),
            _ => format!("{rnum}: payment is {weeks} weeks late: \"{event}\"", $($arg)*),
        }
    });
    ("{} days", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("{} Tage", $($arg)*),
            _ => format!("{} days", $($arg)*),
        }
    });
    ("Responsible: {}", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Verantwortlich: {}", $($arg)*),
            _ => format!("Responsible: {}", $($arg)*),
        }
    });
    ("Archive {}", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Archiviere {}", $($arg)*),
            _ => format!("Archive {}", $($arg)*),
        }
    });
    ("Documentation at: {}", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Dokumentation auf {}", $($arg)*),
            _ => format!("Documentation at: {}", $($arg)*),
        }
    });
    ("{inum }{event:?} on {invoice_date} ({days} days ago) was already invoiced but is still not marked as payed.\nPlease check for incoming payments! You can ask {client} ({mail}).", $($arg:tt)*) => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("{inum }{event:?} wurde am {invoice_date} in Rechnung gestellt (vor {days} Tagen) aber noch nicht als bezahlt markiert.\nBitte kontrolliere den Zahlungseingang und erkundige dich ggf. bei {client} ({mail}).", $($arg)*),
            _ => format!("{inum }{event:?} on {invoice_date} ({days} days ago) was already invoiced but is still not marked as payed.\nPlease check for incoming payments! You can ask {client} ({mail}).", $($arg)*),
        }
    });
    ("Date") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Datum"),
            _ => format!("Date"),
        }
    });
    ("Use a specific template") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Benutze ein bestimmtes Vorlage"),
            _ => format!("Use a specific template"),
        }
    });
    ("Produce an invoice document") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Erzeuge ein Angebot"),
            _ => format!("Produce an invoice document"),
        }
    });
    ("Edit your config") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Anzeigen und Editieren der "),
            _ => format!("Edit your config"),
        }
    });
    ("A template") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Vorlage"),
            _ => format!("A template"),
        }
    });
    ("Show default config") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Anzeigen und Editieren der "),
            _ => format!("Show default config"),
        }
    });
    ("List all projects, ever") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste alle Projekte auf, alle"),
            _ => format!("List all projects, ever"),
        }
    });
    ("List paths to each project file") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Listet Pfade zu Projektdatein"),
            _ => format!("List paths to each project file"),
        }
    });
    ("equals git pop") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("entspricht git pop"),
            _ => format!("equals git pop"),
        }
    });
    ("Archives all projects that can be archived") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Archiviere alle Projekte die archiviert werden können"),
            _ => format!("Archives all projects that can be archived"),
        }
    });
    ("Shows a particular detail") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt ein bestimmtes Detail an"),
            _ => format!("Shows a particular detail"),
        }
    });
    ("equals git stash") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("entspricht git stash"),
            _ => format!("equals git stash"),
        }
    });
    ("Show colors for each project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Fehler mit an"),
            _ => format!("Show colors for each project"),
        }
    });
    ("List projects from that year, archived or not") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste Projekte dieses Jahres auf, ob archiviert oder nicht"),
            _ => format!("List projects from that year, archived or not"),
        }
    });
    ("Show project as JSON") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeige das Projekt als Json"),
            _ => format!("Show project as JSON"),
        }
    });
    ("Open path to current binary instead") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Pfad dieses Programms"),
            _ => format!("Open path to current binary instead"),
        }
    });
    ("The ascii invoicer III") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Der ascii Invoicer III"),
            _ => format!("The ascii invoicer III"),
        }
    });
    ("Show Errors for each project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Fehler mit an"),
            _ => format!("Show Errors for each project"),
        }
    });
    ("Set a value in a project file") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Setzt einen Wert in einer Projekt Datei"),
            _ => format!("Set a value in a project file"),
        }
    });
    ("Print nothing, expect the fields supplied via --details") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Gibt nichts aus, mit Ausnahme der Angaben in --details"),
            _ => format!("Print nothing, expect the fields supplied via --details"),
        }
    });
    ("Show project as yaml") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeige das Projekt als iCal"),
            _ => format!("Show project as yaml"),
        }
    });
    ("Show colors") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste in Farbe"),
            _ => format!("Show colors"),
        }
    });
    ("Pick an archived project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Suche im Archiv"),
            _ => format!("Pick an archived project"),
        }
    });
    ("Open path to templates instead") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Vorlagenverzeichniss"),
            _ => format!("Open path to templates instead"),
        }
    });
    ("Designation") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Bezeichnung"),
            _ => format!("Designation"),
        }
    });
    ("Do not create final output file") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Erzeuge das finale Produkte nicht"),
            _ => format!("Do not create final output file"),
        }
    });
    ("Show a specific config value") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Anzeigen und Editieren der "),
            _ => format!("Show a specific config value"),
        }
    });
    ("Prints version of this tool") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Gibt asciiis Version aus"),
            _ => format!("Prints version of this tool"),
        }
    });
    ("list archived projects of a specific year, defaults to the current year") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste archivierte Projtekte eines bestimmten Jahres auf. Standard: Aktuelles Jahr"),
            _ => format!("list archived projects of a specific year, defaults to the current year"),
        }
    });
    ("Search term, possibly event name") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Suchbegriff oder Eventname"),
            _ => format!("Search term, possibly event name"),
        }
    });
    ("INum") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Rnum"),
            _ => format!("INum"),
        }
    });
    ("Show as csv") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Als CSV anzeigen"),
            _ => format!("Show as csv"),
        }
    });
    ("Search terms to match the project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Suchbegriffe"),
            _ => format!("Search terms to match the project"),
        }
    });
    ("Manually set the end time of the project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Setzt die Endzeit des Projekts manuell"),
            _ => format!("Manually set the end time of the project"),
        }
    });
    ("Create a new project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Rechnung erstellen"),
            _ => format!("Create a new project"),
        }
    });
    ("Produces a CSV report for a given year") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Erzeugt einen CSV Report des gegebenen Jahres"),
            _ => format!("Produces a CSV report for a given year"),
        }
    });
    ("List years in archive") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste Jahre im Archiv"),
            _ => format!("List years in archive"),
        }
    });
    ("Show unpayed wages") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeige unbezahlte "),
            _ => format!("Show unpayed wages"),
        }
    });
    ("Print in csv form") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Ausgabe als CSV"),
            _ => format!("Print in csv form"),
        }
    });
    ("Shows the errors in this project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Fehler im Projekt"),
            _ => format!("Shows the errors in this project"),
        }
    });
    ("cleans changes and untracked files in project folder") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("setzt Änderungen zurück und löscht ungetrackte Datein"),
            _ => format!("cleans changes and untracked files in project folder"),
        }
    });
    ("Show commit logs") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeite Commitlog"),
            _ => format!("Show commit logs"),
        }
    });
    ("Show project as iCal") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeige das Projekt als iCal"),
            _ => format!("Show project as iCal"),
        }
    });
    ("Display a specific project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeige ein Projekt an"),
            _ => format!("Display a specific project"),
        }
    });
    ("Canceled") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Abgesagt"),
            _ => format!("Canceled"),
        }
    });
    ("(experimental) starts interactive shell") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("(experimental) startet eine interaktive shell"),
            _ => format!("(experimental) starts interactive shell"),
        }
    });
    ("Caterer") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Betreuer"),
            _ => format!("Caterer"),
        }
    });
    ("Override the description of the project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Überschreibe die Beschreibung eines Projekts"),
            _ => format!("Override the description of the project"),
        }
    });
    ("Include open tasks") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Aufgaben erzeugen"),
            _ => format!("Include open tasks"),
        }
    });
    ("Project name") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Projektname"),
            _ => format!("Project name"),
        }
    });
    ("Deletes a project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Lösche ein Projekt"),
            _ => format!("Deletes a project"),
        }
    });
    ("Show storage path") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt den Speicherverzeichniss"),
            _ => format!("Show storage path"),
        }
    });
    ("Responsible") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Verantwortlich"),
            _ => format!("Responsible"),
        }
    });
    ("Save changes locally") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Speichert Änderungen lokal"),
            _ => format!("Save changes locally"),
        }
    });
    ("Add extra fields to print for each project listed") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Gibt extra Felder von Projekten mit aus"),
            _ => format!("Add extra fields to print for each project listed"),
        }
    });
    ("Show non-verbose list") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Überschreibe Verbose Einstellung"),
            _ => format!("Show non-verbose list"),
        }
    });
    ("What to put in the field") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Wert"),
            _ => format!("What to put in the field"),
        }
    });
    ("Use a particular template") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Benutze ein bestimmtes Template"),
            _ => format!("Use a particular template"),
        }
    });
    ("Which field to set") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Welches Feld zu setzen?"),
            _ => format!("Which field to set"),
        }
    });
    ("Amount") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Betrag"),
            _ => format!("Amount"),
        }
    });
    ("Edit a template file, use `list --templates` to learn which.") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Bearbeite eine Vorlage. Können mit list --templates aufgelistet werden."),
            _ => format!("Edit a template file, use `list --templates` to learn which."),
        }
    });
    ("Display values in invoice mode") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Werte an (Rechnungs Modules)"),
            _ => format!("Display values in invoice mode"),
        }
    });
    ("Show your name from config") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Anzeigen und Editieren der "),
            _ => format!("Show your name from config"),
        }
    });
    ("The name of the project, duh!") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Der Name des Projekts"),
            _ => format!("The name of the project, duh!"),
        }
    });
    ("List archived projects of a specific year, defaults to the current year") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste archivierte Projtekte eines bestimmten Jahres auf. Standard: Aktuelles Jahr"),
            _ => format!("List archived projects of a specific year, defaults to the current year"),
        }
    });
    ("Shows templates path instead") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Vorlagenverzeichniss"),
            _ => format!("Shows templates path instead"),
        }
    });
    ("Do it against better judgement") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Tu es auch wenn's nicht geht"),
            _ => format!("Do it against better judgement"),
        }
    });
    ("List templates") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste Vorlagen"),
            _ => format!("List templates"),
        }
    });
    ("Override the year") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Überschreibt das Jahr"),
            _ => format!("Override the year"),
        }
    });
    ("Produce an offer document") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Erzeuge ein Angebot"),
            _ => format!("Produce an offer document"),
        }
    });
    ("Sort by :") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Sortiere Ausgabe nach: "),
            _ => format!("Sort by :"),
        }
    });
    ("List all computed data fields that can be used with --details") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Liste mögliche berechnete Felder auf die in --details verwendet werden können."),
            _ => format!("List all computed data fields that can be used with --details"),
        }
    });
    ("Manually set the start time of the project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Setzt die Anfangszeit des Projekts manuell"),
            _ => format!("Manually set the start time of the project"),
        }
    });
    ("list archived projects") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Archivierte Projekte auflisten"),
            _ => format!("list archived projects"),
        }
    });
    ("Opens the online documentation, please read it") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Öffnet die Online Dokumentation, please lies sie!"),
            _ => format!("Opens the online documentation, please read it"),
        }
    });
    ("Manually set the date of the project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Setze das Datum eines Projekts"),
            _ => format!("Manually set the date of the project"),
        }
    });
    ("Move a Project into the archive") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Archiviere ein Project"),
            _ => format!("Move a Project into the archive"),
        }
    });
    ("Edit a specific project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Bearbeite ein bestimmtes Projekt"),
            _ => format!("Edit a specific project"),
        }
    });
    ("Specify the Archiv") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Welches Jahr"),
            _ => format!("Specify the Archiv"),
        }
    });
    ("Show fields in templates that are filled") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Felder an die automatisch gefüllt werden können"),
            _ => format!("Show fields in templates that are filled"),
        }
    });
    ("List Projects") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Projekte auflisten"),
            _ => format!("List Projects"),
        }
    });
    ("Override the manager of the project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Überschreibt den Projektmanager"),
            _ => format!("Override the manager of the project"),
        }
    });
    ("Opposite of simple") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Mehr Details"),
            _ => format!("Opposite of simple"),
        }
    });
    ("Date Format must be DD.MM.YYYY") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Das Datum muss das Format TT.MM.JJJJ haben"),
            _ => format!("Date Format must be DD.MM.YYYY"),
        }
    });
    ("Create config file.") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Rechnung erstellen"),
            _ => format!("Create config file."),
        }
    });
    ("Archives the project, even though it is not completely valid") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Archiviere ein Projekt, selbst wenn es noch nicht vollständig ist"),
            _ => format!("Archives the project, even though it is not completely valid"),
        }
    });
    ("Shows fields that can be filled automatically") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Felder an die automatisch gefüllt werden können"),
            _ => format!("Shows fields that can be filled automatically"),
        }
    });
    ("Overrides the duration of the event") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Überschreibt die Dauer des Events"),
            _ => format!("Overrides the duration of the event"),
        }
    });
    ("Override the configured editor") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Überschreibt das Jahr"),
            _ => format!("Override the configured editor"),
        }
    });
    ("Do not edit the file after creation") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Nach Erstellen nicht editieren"),
            _ => format!("Do not edit the file after creation"),
        }
    });
    ("Move a Project out of the archive") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Verschiebt ein archviertes Projekt zurück ins Arbeitsverzeichnis"),
            _ => format!("Move a Project out of the archive"),
        }
    });
    ("Payed on") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Bezahlt am"),
            _ => format!("Payed on"),
        }
    });
    ("Show the working tree status") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeige den Status des Arbeitsverzeichnisses"),
            _ => format!("Show the working tree status"),
        }
    });
    ("Add file contents to the git-index") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Fügt Ändenderungen zum git-index hinzu"),
            _ => format!("Add file contents to the git-index"),
        }
    });
    ("InvoiceDate") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Rechnungsdatum"),
            _ => format!("InvoiceDate"),
        }
    });
    ("Experimental: open dues") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("(experimentel): zeige offene Posten"),
            _ => format!("Experimental: open dues"),
        }
    });
    ("Show information about the remote") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeit Informationen über den git-remote"),
            _ => format!("Show information about the remote"),
        }
    });
    ("Shows path to created documents instead") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Ausgabeverzeichniss"),
            _ => format!("Shows path to created documents instead"),
        }
    });
    ("Open path to created documents instead") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Ausgabeverzeichniss"),
            _ => format!("Open path to created documents instead"),
        }
    });
    ("Create an Invoice") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Rechnung erstellen"),
            _ => format!("Create an Invoice"),
        }
    });
    ("Show and edit your config") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Anzeigen und Editieren der "),
            _ => format!("Show and edit your config"),
        }
    });
    ("List files that belong to a project") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Datein in diesem Projektverzeichniss"),
            _ => format!("List files that belong to a project"),
        }
    });
    ("Filter selection by field content") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Filtert Ausgabe nach"),
            _ => format!("Filter selection by field content"),
        }
    });
    ("Display values in offer mode") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Zeigt Werte an (Angebots Modules)"),
            _ => format!("Display values in offer mode"),
        }
    });
    ("Creates documents from projects") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Erzeuge Dokumente aus Projekten"),
            _ => format!("Creates documents from projects"),
        }
    });
    ("Open storage path") => ({
        let __guard = ::crowbook_intl_runtime::__get_lang();
        match __guard.as_str() {
            "de" => format!("Öffnet peicherverzeichniss"),
            _ => format!("Open storage path"),
        }
    });
    ($($arg:tt)*) => (format!($($arg)*));
}