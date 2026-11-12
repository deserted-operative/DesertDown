(() => {
    "use strict";

    /*
        MATHS
        only present when the HTML document is written out with maths rendering enabled - the renderer itself knows nothing of Temml
        renders each inline maths span and maths block to MathML with Temml (whose script runs just before this one), which browsers render natively

        the renderer writes maths out as its source, delimiters included (eg. $x^2$, or the $$ lines around a block), so the delimiters are stripped here
        matching the parser, an inline closer is a $ that isn't straight after a \, and a block closer is a $$ after an even number of \s
        (maths left open at the end of its paragraph or the document has no closer)

        trust is off, so Temml refuses commands that could load anything, or set arbitrary attributes or styling (\includegraphics, \style, \class, \id, \data)
        macros defined with \gdef persist from one piece of maths to the next, in document order
        maths Temml can't parse is left as its source, with the error as a tooltip
        a \ref or \eqref to a label that no equation has is shown as ??, with a tooltip
        without scripts, the document is exactly as it would be without maths rendering
    */

    // returns the number of backslashes straight before the index passed in
    const backslashesBefore = (text, index) => {
        let count = 0;

        while (index - count > 0 && text.charCodeAt(index - count - 1) === 92) {
            count++;
        }

        return count;
    };

    // returns the LaTeX content for a math element (block or inline), without delimiters
    const getLatexInElem = (element, isBlock) => {
        const content = element.textContent;
        const delimiterLength = isBlock ? 2 : 1;
        const closerStart = content.length - delimiterLength;

        const hasCloser =
            closerStart >= delimiterLength &&
            content.endsWith(isBlock ? "$$" : "$") &&
            (isBlock
                ? backslashesBefore(content, closerStart) % 2 === 0
                : backslashesBefore(content, closerStart) === 0);

        return content.slice(delimiterLength, hasCloser ? closerStart : content.length);
    };

    // macros defined with \gdef, shared by all the maths in the document
    // init to empty
    const macros = {};

    // use Temml
    for (const element of document.querySelectorAll(".desert-down .math-inline, .desert-down .math-block")) {
        const isBlock = element.classList.contains("math-block");
        const latex = getLatexInElem(element, isBlock);

        // nothing to render
        if (!/\S/.test(latex)) {
            continue;
        }

        // retain source in case of Temml error, as Temml empties the element prior to parsing
        const source = element.textContent;

        try {
            // do NOT trust
            temml.render(latex, element, { displayMode: isBlock, macros, trust: false, throwOnError: true });
        } catch (error) {
            element.textContent = source;
            element.title = error instanceof Error ? error.message : String(error);
        }
    }

    /* fills in \ref and \eqref */
    for (const reference of document.querySelectorAll(".desert-down .tml-ref")) {
        try {
            temml.postProcess(reference.parentElement);
        } catch (error) {
            reference.textContent = "??";
            reference.title = "No equation has this label";
        }
    }
})();
