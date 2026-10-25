# OpenRecall visual direction

The interface is a quiet study desk: Arabic reading, few simultaneous decisions, and an unobstructed question. It opens directly on the library's create action and real collection names. There is no decorative dashboard or sample content in an empty library.

Tokens: ink blue `#101d30` canvas, slate blue `#192b42` surfaces, soft white `#edf2f7` primary text, mist `#b8c9db` secondary text, cornflower `#96bdff` actions/focus, rose `#ffb4bd` destructive/error text. A narrow indigo vertical rail anchors the library/deck hierarchy. Its location follows RTL; content aligns to the inline start. The review canvas uses a generous single column, with question and answer blocks separated by spacing rather than a grid of cards.

Use locally bundled Noto Sans Arabic Variable for both Arabic navigation and its supported Latin forms, with system fallbacks for other card languages. Body 18px, line height 1.85; H1 32px, H2 24px, labels 16px. Widths remain below 74 characters for study content. Buttons have at least 44px target height. Native dialogs provide the modal focus boundary; explicit focus moves to page/question/answer headings after navigation.

Library layout: navigation rail | create action and H1 | collection list with counts and named actions. A collection page has its review and create buttons visible, followed by closed disclosure sections for cards, statistics, and scheduling. The distinctive element is the quiet blue study column, not bright gradients or ornamental metric tiles. The initial grid proposal was reduced to a collection list: this handles long Arabic names, screen magnification and keyboard reading order more consistently.

Only user-triggered disclosure/dialog transitions are allowed; reduced-motion preference removes them. Text content always uses `dir=auto`, preserves whitespace and is rendered as text nodes. Automated accessibility tests supplement, but do not replace, the user's human NVDA test.
