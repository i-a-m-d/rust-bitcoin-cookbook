(function () {
    // Change this one number to control how many heading levels appear
    // in the left sidebar's "On This Page" section.
    const sidebarHeadingLevels = 1;
    const normalizedLevels = Math.max(0, Math.min(sidebarHeadingLevels, 5));

    document.documentElement.dataset.sidebarHeadingLevels = String(normalizedLevels);
})();
