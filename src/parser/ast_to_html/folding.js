(() => {
    "use strict";

    // get the level of a heading (1 to 6) if the element is a heading, or 0 if it's not
    const getHeadingLevel = (element) => {
        const tag = element.tagName;
        const digit = tag.charCodeAt(1) - 48; // tags are h1, h2, h3, h4, h5, h6 - so inspect second digit

        return tag.length === 2 && tag.charCodeAt(0) === 72 && digit >= 1 && digit <= 6 ? digit : 0;
    };

    // paragraphs consisting of blank lines considered as spacing rather than content
    const isBlank = (element) =>
        element.tagName === "P" && element.childElementCount === 0 && !/\S/.test(element.textContent);

    // takes an element, returns true if there's (non-whitespace) content at or after it and before the end or next heading of a level higher than what's passed in
    const hasContentFrom = (element, level) => {
        for (; element !== null; element = element.nextElementSibling) {
            // getHeadingLevel will return 0 for not a (valid) heading
            const elementLevel = getHeadingLevel(element);

            if (elementLevel !== 0 && elementLevel <= level) {
                return false;
            }

            // ignore whitespace
            if (!isBlank(element)) {
                return true;
            }
        }

        return false;
    };

    // returns the last element in a given heading section that isn't blank (or null if there isn't one)
    const lastContentInSection = (heading) => {
        const level = getHeadingLevel(heading);
        let lastContent = null;

        for (let element = heading.nextElementSibling; element !== null; element = element.nextElementSibling) {
            const elementLevel = getHeadingLevel(element);

            if (elementLevel !== 0 && elementLevel <= level) {
                break;
            }

            if (!isBlank(element)) {
                lastContent = element;
            }
        }

        return lastContent;
    };

    // returns the first block of a list item (after any checkbox), or null if it has none
    const firstBlockOfListItem = (listItem) => {
        const firstBlock = listItem.firstElementChild;

        // ternary deals with checkboxes
        return firstBlock !== null && firstBlock.tagName === "INPUT" ? firstBlock.nextElementSibling : firstBlock;
    };

    // returns whether a list item's first block is a paragraph with continuations (ie. a line after the first that isn't blank)
    const hasContinuations = (firstBlock) => firstBlock.tagName === "P" && /\n\s*\S/.test(firstBlock.textContent);

    /*
        returns the last of the indent blocks straight after a list (null if there aren't any)
        (required as indented blocks within list items aren't placed within the HTML list item elements)
        
        blank lines between indent blocks are considered nested content, but those after the last aren't, as they aren't indented
    */
    const lastIndentAfterList = (list) => {
        let lastIndent = null;

        for (let node = list.nextSibling; node !== null; node = node.nextSibling) {
            if (node.nodeType === Node.COMMENT_NODE) {
                // folding break terminates nested content
                if (node.data === "folding break") {
                    break;
                }

                continue;
            }

            if (node.nodeType !== Node.ELEMENT_NODE) {
                continue;
            }

            if (node.tagName === "DIV" && node.classList.contains("indent")) {
                lastIndent = node;
            } else if (!isBlank(node)) {
                break;
            }
        }

        return lastIndent;
    };

    // takes a list item, finds the list block, runs the above function
    const lastIndentAfterListItem = (listItem) =>
        listItem.nextElementSibling === null ? lastIndentAfterList(listItem.parentElement) : null;

    const markNode = (node, className) => {
        if (node.nodeType === Node.ELEMENT_NODE) {
            node.classList.add(className);
        } else if (node.nodeType === Node.TEXT_NODE) {
            const wrapper = document.createElement("span");

            wrapper.className = className;
            node.replaceWith(wrapper);
            wrapper.append(node);
        }
    };

    const markOutwards = (paragraph, node, forwards, className) => {
        let parent = node.parentNode;

        for (; ;) {
            while (node !== null) {
                const next = forwards ? node.nextSibling : node.previousSibling;

                markNode(node, className);
                node = next;
            }

            if (parent === paragraph) {
                break;
            }

            node = forwards ? parent.nextSibling : parent.previousSibling;
            parent = parent.parentNode;
        }
    };

    const markedParagraphs = new WeakSet();

    const markContinuations = (paragraph) => {
        if (markedParagraphs.has(paragraph)) {
            return;
        }

        markedParagraphs.add(paragraph);

        const walker = document.createTreeWalker(paragraph, NodeFilter.SHOW_TEXT);
        let newlineIndex = -1;

        while (walker.nextNode() !== null && (newlineIndex = walker.currentNode.data.indexOf("\n")) === -1) { }

        if (newlineIndex === -1) {
            return;
        }

        markOutwards(paragraph, walker.currentNode.splitText(newlineIndex), true, "dd-fold-rest");
    };

    const trailingBlankLinesOffset = (paragraph) => {
        const text = paragraph.textContent;
        const lastLineEnd = text.indexOf("\n", text.trimEnd().length);

        return lastLineEnd !== -1 && lastLineEnd + 1 < text.length ? lastLineEnd + 1 : -1;
    };

    const leadMarkedParagraphs = new WeakSet();

    const markLead = (paragraph, offset) => {
        if (leadMarkedParagraphs.has(paragraph)) {
            return;
        }

        leadMarkedParagraphs.add(paragraph);

        const walker = document.createTreeWalker(paragraph, NodeFilter.SHOW_TEXT);
        let nodeStart = 0;

        while (walker.nextNode() !== null && nodeStart + walker.currentNode.length < offset) {
            nodeStart += walker.currentNode.length;
        }

        const leadEnd = walker.currentNode;

        if (offset - nodeStart < leadEnd.length) {
            leadEnd.splitText(offset - nodeStart);
        }

        markOutwards(paragraph, leadEnd, false, "dd-fold-lead");
    };

    const hideLastContent = (element) => {
        const offset = element.tagName === "P" ? trailingBlankLinesOffset(element) : -1;

        if (offset === -1) {
            element.classList.add("dd-fold-hidden");
            return;
        }

        markLead(element, offset);
        element.classList.add("dd-fold-keep-trail");
    };

    const LABEL_TEXT_MAX_LENGTH = 80;

    const toggleLabel = (action, text) => {
        const line = text.split("\n", 1)[0].replace(/\s+/g, " ").trim();

        if (line === "") {
            return action;
        }

        return action + ": " + (line.length > LABEL_TEXT_MAX_LENGTH ? line.slice(0, LABEL_TEXT_MAX_LENGTH - 1).trimEnd() + "\u2026" : line);
    };

    const createToggle = (kind, label) => {
        const toggle = document.createElement("button");

        toggle.type = "button";
        toggle.className = "dd-fold dd-fold-" + kind;
        toggle.setAttribute("aria-label", label);
        toggle.setAttribute("aria-expanded", "true");

        return toggle;
    };

    const toggleHeading = (heading, folded) => {
        if (folded) {
            const lastContent = lastContentInSection(heading);

            for (let element = heading.nextElementSibling; element !== null; element = element.nextElementSibling) {
                if (element === lastContent) {
                    hideLastContent(element);
                    break;
                }

                element.classList.add("dd-fold-hidden");
            }

            return;
        }

        const level = getHeadingLevel(heading);

        let hiddenUntil = null;

        for (let element = heading.nextElementSibling; element !== null; element = element.nextElementSibling) {
            if (hiddenUntil !== null) {
                if (element === hiddenUntil) {
                    // this heading's fold may have hidden it fully - hide it as the still folded heading did
                    element.classList.remove("dd-fold-hidden");
                    hideLastContent(element);
                    hiddenUntil = null;
                }

                continue;
            }

            const elementLevel = getHeadingLevel(element);

            if (elementLevel !== 0 && elementLevel <= level) {
                break;
            }

            element.classList.remove("dd-fold-hidden", "dd-fold-keep-trail");

            if (elementLevel !== 0 && element.classList.contains("dd-folded")) {
                hiddenUntil = lastContentInSection(element);
            }
        }
    };

    const foldsKind = (root, kind) => {
        const kinds = root.getAttribute("data-dd-fold");

        return kinds === null || kinds.split(/\s+/).includes(kind);
    };

    const setUpFolding = (root) => {
        for (const heading of foldsKind(root, "headings") ? root.querySelectorAll("h1,h2,h3,h4,h5,h6") : []) {
            const parent = heading.parentElement;

            if (parent.tagName === "LI" && firstBlockOfListItem(parent) === heading) {
                continue;
            }

            if (hasContentFrom(heading.nextElementSibling, getHeadingLevel(heading))) {
                heading.prepend(createToggle("heading", toggleLabel("Toggle section", heading.textContent)));
            }
        }

        for (const listItem of foldsKind(root, "list-items") ? root.querySelectorAll("li") : []) {
            const firstBlock = firstBlockOfListItem(listItem);

            if (
                firstBlock !== null &&
                (hasContinuations(firstBlock) ||
                    hasContentFrom(firstBlock.nextElementSibling, 0) ||
                    lastIndentAfterListItem(listItem) !== null)
            ) {
                const toggle = createToggle("list-item", toggleLabel("Toggle list item", firstBlock.textContent));

                if (listItem.parentElement.tagName === "OL") {
                    toggle.setAttribute("data-marker", (listItem.getAttribute("value") ?? "") + ".");
                }

                firstBlock.before(toggle);
            }
        }

        for (const title of foldsKind(root, "callouts") ? root.querySelectorAll(".callout > .callout-title") : []) {
            if (hasContentFrom(title.nextElementSibling, 0)) {
                title.prepend(createToggle("callout", toggleLabel("Toggle callout", title.textContent)));
            }
        }

        root.addEventListener("click", (event) => {
            const toggle = event.target;

            if (!(toggle instanceof HTMLButtonElement) || !toggle.classList.contains("dd-fold")) {
                return;
            }

            const folded = toggle.getAttribute("aria-expanded") === "true";
            toggle.setAttribute("aria-expanded", folded ? "false" : "true");

            if (toggle.classList.contains("dd-fold-heading")) {
                const heading = toggle.parentElement;

                heading.classList.toggle("dd-folded", folded);
                toggleHeading(heading, folded);
            } else if (toggle.classList.contains("dd-fold-list-item")) {
                if (folded) {
                    const firstBlock = toggle.nextElementSibling;

                    if (hasContinuations(firstBlock)) {
                        markContinuations(firstBlock);
                    }
                }

                const listItem = toggle.parentElement;

                listItem.classList.toggle("dd-folded", folded);

                const lastIndent = lastIndentAfterListItem(listItem);

                if (lastIndent !== null) {
                    for (let element = listItem.parentElement.nextElementSibling; ; element = element.nextElementSibling) {
                        element.classList.toggle("dd-fold-after-list", folded);

                        if (element === lastIndent) {
                            break;
                        }
                    }
                }
            } else {
                toggle.parentElement.parentElement.classList.toggle("dd-folded", folded);
            }
        });
    };

    for (const root of document.querySelectorAll(".desert-down")) {
        setUpFolding(root);
    }
})();
