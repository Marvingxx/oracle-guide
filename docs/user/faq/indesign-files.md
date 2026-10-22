---
title: Can Oracle Brand Guide open InDesign files?
description: Oracle Brand Guide opens and exports InDesign Markup (.idml) files. The Open dialog does not list InDesign's own .indd format.
order: 30
verified: 0.7.1
---

Yes for IDML, the InDesign Markup format. Oracle Brand Guide can open `.idml` files and export them with **File**, then **Export IDML…**.

The **Open…** dialog lists `.oracle-guide` and `.idml` files only, so it does not offer InDesign's own `.indd` files. See [Open a Oracle Brand Guide or InDesign (IDML) file](../guides/open-a-file.md).

IDML import is not complete. Open issues in the tracker report that custom text-wrap boundaries ([#451](https://github.com/Marvingxx/oracle-guide/issues/451)) and saved image clipping paths ([#450](https://github.com/Marvingxx/oracle-guide/issues/450)) are lost when importing IDML, and that bullets and numbers in lists show as missing-font magenta in some templates ([#110](https://github.com/Marvingxx/oracle-guide/issues/110)). Check your layout after importing.

Oracle Brand Guide can also export PDF (including PDF/X-4 and PDF/A-2b), PNG and EPUB. See [Export a PDF](../guides/export-a-pdf.md).
