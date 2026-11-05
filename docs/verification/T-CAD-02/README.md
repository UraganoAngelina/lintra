# Evidenze T-CAD-02

preliminary-format e preliminary-golden-version conservano i gate falliti,
non sono PASS. La regressione di isolamento è conservata prima del fix e
nella suite mirata dopo il fix. Il file golden-version-changes documenta
soltanto il cambio di metadato rc.2→rc.3 in tre summary SIOC, con hash prima/dopo.

Il gate locale conclusivo viene copiato qui dopo completamento e verifica degli
hash sorgenti. La CI nativa successiva è un gate distinto sul commit pubblicato.
Nessun artefatto del prototipo DEC-17 viene trasferito nel runtime.
