// Y++ IDE — Graphical shell/IDE for the Y++ language
package ypp;

import javax.swing.*;
import javax.swing.border.*;
import javax.swing.event.*;
import javax.swing.filechooser.FileNameExtensionFilter;
import javax.swing.text.*;
import javax.swing.tree.*;
import java.awt.*;
import java.awt.event.*;
import java.awt.image.BufferedImage;
import java.io.*;
import java.nio.file.*;
import java.util.*;
import java.util.List;
import java.util.regex.*;

public class YppIDE extends JFrame {

    static final Color BG_EDITOR    = new Color(0x1E1E1E);
    static final Color BG_PANEL     = new Color(0x252526);
    static final Color BG_TOOLBAR   = new Color(0x333333);
    static final Color BG_CONSOLE   = new Color(0x0F0F0F);
    static final Color BG_GUTTER    = new Color(0x1A1A1A);
    static final Color BG_SIDEBAR   = new Color(0x1E1E2E);
    static final Color FG_TEXT      = new Color(0xD4D4D4);
    static final Color FG_GUTTER    = new Color(0x858585);
    static final Color FG_KEYWORD   = new Color(0x569CD6);
    static final Color FG_TYPE      = new Color(0x4EC9B0);
    static final Color FG_STRING    = new Color(0xCE9178);
    static final Color FG_NUMBER    = new Color(0xB5CEA8);
    static final Color FG_COMMENT   = new Color(0x6A9955);
    static final Color FG_CAST      = new Color(0xDCDCAA);
    static final Color FG_FUNC      = new Color(0xDDDD33);
    static final Color ACCENT       = new Color(0x007ACC);
    static final Color BTN_RUN      = new Color(0x23D18B);
    static final Color BTN_RUN_FG   = new Color(0x0F0F0F);
    static final Color CONSOLE_OUT  = new Color(0xD4D4D4);
    static final Color CONSOLE_ERR  = new Color(0xF48771);
    static final Color CONSOLE_SYS  = new Color(0x569CD6);
    static final Color BORDER_COLOR = new Color(0x3C3C3C);
    static final Color SIDEBAR_SEL  = new Color(0x094771);

    static final Font EDITOR_FONT = new Font("Consolas", Font.PLAIN, 15);
    static final Font CONSOLE_FONT= new Font("Consolas", Font.PLAIN, 13);
    static final Font UI_FONT     = new Font("Segoe UI",  Font.PLAIN, 13);
    static final Font UI_BOLD     = new Font("Segoe UI",  Font.BOLD,  13);
    static final Font TREE_FONT   = new Font("Segoe UI",  Font.PLAIN, 12);

    private final JTextPane   editor      = new JTextPane();
    private final JTextPane   console     = new JTextPane();
    private final JLabel      statusLabel = new JLabel(" Ln 1, Col 1");
    private final JLabel      fileLabel   = new JLabel(" untitled.ypp");
    private       File        currentFile = null;
    private       boolean     dirty       = false;

    private File             projectRoot = null;
    private DefaultTreeModel treeModel;
    private JTree            fileTree;

    // Interactive terminal input
    private final java.util.concurrent.SynchronousQueue<String> inputQueue = new java.util.concurrent.SynchronousQueue<>();
    private JTextField inputField;
    private JLabel     inputPromptLabel;

    private final IntelliSense intelliSense = new IntelliSense();
    private final LineGutter   gutter;

    public YppIDE() {
        super("Y++ IDE");
        setDefaultCloseOperation(DO_NOTHING_ON_CLOSE);
        setSize(1400, 850);
        setMinimumSize(new Dimension(1000, 600));
        setLocationRelativeTo(null);
        setIconImage(makeIcon());

        UIManager.put("Panel.background", BG_PANEL);
        UIManager.put("SplitPane.background", BG_PANEL);
        UIManager.put("SplitPane.dividerSize", 4);

        editor.setFont(EDITOR_FONT);
        editor.setBackground(BG_EDITOR);
        editor.setForeground(FG_TEXT);
        editor.setCaretColor(new Color(0xAEAFAD));
        editor.setSelectionColor(new Color(0x264F78));
        editor.setSelectedTextColor(FG_TEXT);
        editor.setDocument(new DefaultStyledDocument());
        editor.putClientProperty("caretAspectRatio", 0.08f);
        insertTemplate();

        editor.getDocument().addDocumentListener(new DocumentListener() {
            private final javax.swing.Timer timer =
                new javax.swing.Timer(120, e -> { rehighlight(); updateStatus(); });
            { timer.setRepeats(false); }
            public void insertUpdate(DocumentEvent e)  { dirty=true; updateTitle(); timer.restart(); intelliSense.onDocumentChange(); }
            public void removeUpdate(DocumentEvent e)  { dirty=true; updateTitle(); timer.restart(); intelliSense.onDocumentChange(); }
            public void changedUpdate(DocumentEvent e) {}
        });

        editor.addKeyListener(new KeyAdapter() {
            @Override public void keyPressed(KeyEvent e) {
                if (intelliSense.isVisible()) {
                    if (e.getKeyCode()==KeyEvent.VK_DOWN)  { intelliSense.selectNext(); e.consume(); return; }
                    if (e.getKeyCode()==KeyEvent.VK_UP)    { intelliSense.selectPrev(); e.consume(); return; }
                    if (e.getKeyCode()==KeyEvent.VK_ENTER
                     || e.getKeyCode()==KeyEvent.VK_TAB)   { intelliSense.applySelected(); e.consume(); return; }
                    if (e.getKeyCode()==KeyEvent.VK_ESCAPE){ intelliSense.hide(); e.consume(); return; }
                }
                if (e.getKeyCode()==KeyEvent.VK_ENTER && !e.isShiftDown()) { e.consume(); insertSmartNewline(); }
            }
            @Override public void keyTyped(KeyEvent e) {
                if (e.getKeyChar()=='{') { e.consume(); insertAutoCloseBrace(); }
            }
        });
        intelliSense.attach(editor);

        gutter = new LineGutter(editor);

        JScrollPane editorScroll = new JScrollPane(editor);
        editorScroll.setRowHeaderView(gutter);
        editorScroll.setBorder(BorderFactory.createEmptyBorder());
        editorScroll.getViewport().setBackground(BG_EDITOR);
        editorScroll.setBackground(BG_EDITOR);

        console.setFont(CONSOLE_FONT);
        console.setBackground(BG_CONSOLE);
        console.setForeground(CONSOLE_OUT);
        console.setEditable(false);
        console.setCaretColor(BG_CONSOLE);

        JScrollPane consoleScroll = new JScrollPane(console);
        consoleScroll.setBorder(BorderFactory.createEmptyBorder());
        consoleScroll.getViewport().setBackground(BG_CONSOLE);

        JLabel consoleHeader = styledLabel(" OUTPUT", UI_BOLD, CONSOLE_SYS);
        consoleHeader.setBorder(new MatteBorder(0,0,1,0,BORDER_COLOR));
        consoleHeader.setOpaque(true);
        consoleHeader.setBackground(BG_PANEL);
        consoleHeader.setPreferredSize(new Dimension(0, 26));

        JButton clearBtn = flatButton("Clear", BTN_RUN, BTN_RUN_FG);
        clearBtn.setPreferredSize(new Dimension(60, 22));
        clearBtn.setFont(new Font("Segoe UI", Font.BOLD, 11));
        clearBtn.addActionListener(e -> clearConsole());

        JPanel consoleTopBar = new JPanel(new BorderLayout());
        consoleTopBar.setBackground(BG_PANEL);
        consoleTopBar.setBorder(new MatteBorder(0,0,1,0,BORDER_COLOR));
        consoleTopBar.add(consoleHeader, BorderLayout.CENTER);
        JPanel clearWrap = new JPanel(new FlowLayout(FlowLayout.RIGHT, 6, 2));
        clearWrap.setOpaque(false);
        clearWrap.add(clearBtn);
        consoleTopBar.add(clearWrap, BorderLayout.EAST);

        JPanel consolePanel = new JPanel(new BorderLayout());
        consolePanel.setBackground(BG_CONSOLE);
        consolePanel.add(consoleTopBar, BorderLayout.NORTH);
        consolePanel.add(consoleScroll, BorderLayout.CENTER);

        // -- Interactive input bar --
        inputPromptLabel = new JLabel(">");
        inputPromptLabel.setFont(CONSOLE_FONT);
        inputPromptLabel.setForeground(new Color(0x6A9955));
        inputPromptLabel.setBorder(BorderFactory.createEmptyBorder(0, 6, 0, 4));

        inputField = new JTextField();
        inputField.setFont(CONSOLE_FONT);
        inputField.setBackground(new Color(0x1A1A2E));
        inputField.setForeground(new Color(0xD4D4D4));
        inputField.setCaretColor(new Color(0xAEAFAD));
        inputField.setBorder(BorderFactory.createEmptyBorder(2, 4, 2, 4));
        inputField.setEnabled(false);
        inputField.addActionListener(ev -> {
            String text = inputField.getText();
            inputField.setText("");
            inputField.setEnabled(false);
            inputPromptLabel.setForeground(new Color(0x6A9955));
            try { inputQueue.put(text); } catch (InterruptedException ignored) {}
        });

        JPanel inputBar = new JPanel(new BorderLayout());
        inputBar.setBackground(new Color(0x1A1A2E));
        inputBar.setBorder(new MatteBorder(1, 0, 0, 0, BORDER_COLOR));
        inputBar.add(inputPromptLabel, BorderLayout.WEST);
        inputBar.add(inputField, BorderLayout.CENTER);

        consolePanel.add(inputBar, BorderLayout.SOUTH);

        JSplitPane vertSplit = new JSplitPane(JSplitPane.VERTICAL_SPLIT, editorScroll, consolePanel);
        vertSplit.setResizeWeight(0.72);
        vertSplit.setDividerSize(5);
        vertSplit.setBackground(BORDER_COLOR);

        JPanel sidebar = buildSidebar();

        JSplitPane horizSplit = new JSplitPane(JSplitPane.HORIZONTAL_SPLIT, sidebar, vertSplit);
        horizSplit.setDividerLocation(220);
        horizSplit.setDividerSize(4);
        horizSplit.setResizeWeight(0.0);
        horizSplit.setBackground(BORDER_COLOR);

        setLayout(new BorderLayout());
        add(buildToolbar(), BorderLayout.NORTH);
        add(horizSplit,     BorderLayout.CENTER);
        add(buildStatusBar(), BorderLayout.SOUTH);

        bindKeys();
        addWindowListener(new WindowAdapter() { @Override public void windowClosing(WindowEvent e) { confirmAndClose(); } });
        SwingUtilities.invokeLater(this::rehighlight);
        setVisible(true);
        editor.requestFocusInWindow();
    }

    // ── Smart editor helpers ──────────────────────────────────────

    private void insertSmartNewline() {
        try {
            Document doc = editor.getDocument();
            int caretPos = editor.getCaretPosition();
            Element root = doc.getDefaultRootElement();
            Element line = root.getElement(root.getElementIndex(caretPos));
            String lineText = doc.getText(line.getStartOffset(), line.getEndOffset() - line.getStartOffset());
            StringBuilder indent = new StringBuilder();
            for (char c : lineText.toCharArray()) { if (c==' '||c=='\t') indent.append(c); else break; }
            if (lineText.stripTrailing().endsWith("{")) indent.append("    ");
            doc.insertString(caretPos, "\n" + indent, null);
        } catch (BadLocationException ignored) {}
    }

    private void insertAutoCloseBrace() {
        try {
            Document doc = editor.getDocument();
            int caretPos = editor.getCaretPosition();
            Element root = doc.getDefaultRootElement();
            Element line = root.getElement(root.getElementIndex(caretPos));
            String lineText = doc.getText(line.getStartOffset(), line.getEndOffset() - line.getStartOffset());
            StringBuilder indent = new StringBuilder();
            for (char c : lineText.toCharArray()) { if (c==' '||c=='\t') indent.append(c); else break; }
            String inner = indent + "    ";
            doc.insertString(caretPos, "{\n" + inner + "\n" + indent + "}", null);
            editor.setCaretPosition(caretPos + 2 + inner.length());
        } catch (BadLocationException ignored) {}
    }

    // ── File Explorer Sidebar ─────────────────────────────────────

    private JPanel buildSidebar() {
        JLabel header = styledLabel(" EXPLORER", UI_BOLD, FG_KEYWORD);
        header.setOpaque(true);
        header.setBackground(BG_SIDEBAR);
        header.setPreferredSize(new Dimension(0, 28));
        header.setBorder(new MatteBorder(0,0,1,0,BORDER_COLOR));

        JButton openFolderBtn = sidebarBtn("Open Folder");
        JButton newFileBtn    = sidebarBtn("New File");
        JButton newFolderBtn  = sidebarBtn("New Folder");
        openFolderBtn.addActionListener(e -> openFolder());
        newFileBtn   .addActionListener(e -> sidebarNewFile());
        newFolderBtn .addActionListener(e -> sidebarNewFolder());

        JPanel btnRow = new JPanel(new FlowLayout(FlowLayout.LEFT, 4, 4));
        btnRow.setBackground(BG_SIDEBAR);
        btnRow.add(openFolderBtn); btnRow.add(newFileBtn); btnRow.add(newFolderBtn);
        btnRow.setBorder(new MatteBorder(0,0,1,0,BORDER_COLOR));

        DefaultMutableTreeNode emptyRoot = new DefaultMutableTreeNode("No folder open");
        treeModel = new DefaultTreeModel(emptyRoot);
        fileTree  = new JTree(treeModel);
        fileTree.setFont(TREE_FONT);
        fileTree.setBackground(BG_SIDEBAR);
        fileTree.setForeground(FG_TEXT);
        fileTree.setRowHeight(22);
        fileTree.setRootVisible(true);
        fileTree.setShowsRootHandles(true);
        styleTree(fileTree);

        fileTree.addMouseListener(new MouseAdapter() {
            @Override public void mouseClicked(MouseEvent e) {
                TreePath path = fileTree.getPathForLocation(e.getX(), e.getY());
                if (path == null) return;
                DefaultMutableTreeNode node = (DefaultMutableTreeNode) path.getLastPathComponent();
                if (e.getClickCount() == 2) openTreeNode(node);
                if (SwingUtilities.isRightMouseButton(e)) { fileTree.setSelectionPath(path); showTreeContextMenu(e, node); }
            }
        });

        JScrollPane treeScroll = new JScrollPane(fileTree);
        treeScroll.setBorder(BorderFactory.createEmptyBorder());
        treeScroll.getViewport().setBackground(BG_SIDEBAR);

        JPanel top = new JPanel(new BorderLayout());
        top.setBackground(BG_SIDEBAR);
        top.add(header, BorderLayout.NORTH);
        top.add(btnRow, BorderLayout.SOUTH);

        JPanel panel = new JPanel(new BorderLayout());
        panel.setBackground(BG_SIDEBAR);
        panel.setPreferredSize(new Dimension(220, 0));
        panel.add(top,       BorderLayout.NORTH);
        panel.add(treeScroll, BorderLayout.CENTER);
        panel.setBorder(new MatteBorder(0,0,0,1,BORDER_COLOR));
        return panel;
    }

    private JButton sidebarBtn(String text) {
        JButton btn = new JButton(text);
        btn.setFont(new Font("Segoe UI", Font.PLAIN, 11));
        btn.setForeground(FG_TEXT);
        btn.setBackground(new Color(0x2D2D3E));
        btn.setOpaque(true);
        btn.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(BORDER_COLOR,1),
            BorderFactory.createEmptyBorder(2,6,2,6)));
        btn.setFocusPainted(false);
        btn.setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        btn.addMouseListener(new MouseAdapter() {
            public void mouseEntered(MouseEvent e) { btn.setBackground(new Color(0x3A3A55)); }
            public void mouseExited(MouseEvent e)  { btn.setBackground(new Color(0x2D2D3E)); }
        });
        return btn;
    }

    private void styleTree(JTree tree) {
        tree.setCellRenderer(new DefaultTreeCellRenderer() {
            @Override public Component getTreeCellRendererComponent(JTree t, Object value,
                    boolean sel, boolean exp, boolean leaf, int row, boolean hasFocus) {
                super.getTreeCellRendererComponent(t, value, sel, exp, leaf, row, hasFocus);
                setFont(TREE_FONT);
                setForeground(FG_TEXT);
                setBackground(sel ? SIDEBAR_SEL : BG_SIDEBAR);
                setOpaque(true);
                setBorderSelectionColor(sel ? SIDEBAR_SEL : BG_SIDEBAR);
                DefaultMutableTreeNode node = (DefaultMutableTreeNode) value;
                Object uo = node.getUserObject();
                if (uo instanceof File f) setText((f.isDirectory() ? "[F] " : "    ") + f.getName());
                else setText(String.valueOf(uo));
                return this;
            }
        });
    }

    private void openFolder() {
        JFileChooser fc = new JFileChooser(projectRoot != null ? projectRoot : new File("."));
        fc.setFileSelectionMode(JFileChooser.DIRECTORIES_ONLY);
        fc.setDialogTitle("Open Folder");
        if (fc.showOpenDialog(this) != JFileChooser.APPROVE_OPTION) return;
        projectRoot = fc.getSelectedFile();
        refreshFileTree();
    }

    private void refreshFileTree() {
        if (projectRoot == null) return;
        treeModel.setRoot(buildTreeNode(projectRoot));
        treeModel.reload();
        fileTree.expandRow(0);
    }

    private DefaultMutableTreeNode buildTreeNode(File file) {
        DefaultMutableTreeNode node = new DefaultMutableTreeNode(file);
        if (file.isDirectory()) {
            File[] children = file.listFiles();
            if (children != null) {
                Arrays.sort(children, (a, b) -> {
                    if (a.isDirectory() != b.isDirectory()) return a.isDirectory() ? -1 : 1;
                    return a.getName().compareToIgnoreCase(b.getName());
                });
                for (File child : children) node.add(buildTreeNode(child));
            }
        }
        return node;
    }

    private void openTreeNode(DefaultMutableTreeNode node) {
        Object uo = node.getUserObject();
        if (!(uo instanceof File f) || f.isDirectory()) return;
        if (!confirmDiscardChanges()) return;
        try {
            editor.setText(Files.readString(f.toPath()));
            currentFile = f; dirty = false; updateTitle();
            SwingUtilities.invokeLater(this::rehighlight);
        } catch (IOException ex) {
            JOptionPane.showMessageDialog(this, "Could not open:\n"+ex.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
        }
    }

    private void showTreeContextMenu(MouseEvent e, DefaultMutableTreeNode node) {
        Object uo   = node.getUserObject();
        File target = (uo instanceof File f) ? f : projectRoot;
        File dir    = (target!=null && target.isDirectory()) ? target
                    : (target!=null ? target.getParentFile() : projectRoot);

        JPopupMenu menu = new JPopupMenu();
        menu.setBackground(BG_PANEL);

        JMenuItem newFile = styledMenuItem("New File");
        JMenuItem newDir  = styledMenuItem("New Folder");
        JMenuItem newPkg  = styledMenuItem("New Package");
        JMenuItem rename  = styledMenuItem("Rename");
        JMenuItem delete  = styledMenuItem("Delete");

        newFile.addActionListener(ae -> {
            String name = JOptionPane.showInputDialog(this, "File name:", "New File", JOptionPane.PLAIN_MESSAGE);
            if (name!=null && !name.isBlank()) {
                File nf = new File(dir, name.endsWith(".ypp") ? name : name+".ypp");
                try { nf.createNewFile(); refreshFileTree(); } catch (IOException ex) { showErr(ex); }
            }
        });
        newDir.addActionListener(ae -> {
            String name = JOptionPane.showInputDialog(this, "Folder name:", "New Folder", JOptionPane.PLAIN_MESSAGE);
            if (name!=null && !name.isBlank()) { new File(dir, name).mkdirs(); refreshFileTree(); }
        });
        newPkg.addActionListener(ae -> {
            String name = JOptionPane.showInputDialog(this, "Package name:", "New Package", JOptionPane.PLAIN_MESSAGE);
            if (name!=null && !name.isBlank()) { 
                new File(dir, name).mkdirs(); 
                refreshFileTree();
                try {
                    editor.getDocument().insertString(0, "Import " + name + " *\n", null);
                } catch (BadLocationException ex) {
                    // ignore
                }
            }
        });
        rename.addActionListener(ae -> {
            if (target==null) return;
            String name = JOptionPane.showInputDialog(this, "New name:", target.getName());
            if (name!=null && !name.isBlank()) { target.renameTo(new File(target.getParentFile(), name)); refreshFileTree(); }
        });
        delete.addActionListener(ae -> {
            if (target==null) return;
            int r = JOptionPane.showConfirmDialog(this, "Delete \""+target.getName()+"\"?",
                "Delete", JOptionPane.YES_NO_OPTION, JOptionPane.WARNING_MESSAGE);
            if (r==JOptionPane.YES_OPTION) { deleteRecursive(target); refreshFileTree(); }
        });

        menu.add(newFile); menu.add(newDir); menu.add(newPkg); menu.addSeparator(); menu.add(rename); menu.add(delete);
        menu.show(fileTree, e.getX(), e.getY());
    }

    private JMenuItem styledMenuItem(String text) {
        JMenuItem item = new JMenuItem(text);
        item.setBackground(BG_PANEL); item.setForeground(FG_TEXT); item.setFont(UI_FONT);
        return item;
    }

    private void deleteRecursive(File f) {
        if (f.isDirectory()) for (File c : Objects.requireNonNull(f.listFiles())) deleteRecursive(c);
        f.delete();
    }

    private void sidebarNewFile() {
        File dir = projectRoot!=null ? projectRoot : new File(".");
        String name = JOptionPane.showInputDialog(this, "File name:", "New File", JOptionPane.PLAIN_MESSAGE);
        if (name!=null && !name.isBlank()) {
            File nf = new File(dir, name.endsWith(".ypp") ? name : name+".ypp");
            try { nf.createNewFile(); refreshFileTree(); } catch (IOException ex) { showErr(ex); }
        }
    }

    private void sidebarNewFolder() {
        File dir = projectRoot!=null ? projectRoot : new File(".");
        String name = JOptionPane.showInputDialog(this, "Folder name:", "New Folder", JOptionPane.PLAIN_MESSAGE);
        if (name!=null && !name.isBlank()) { new File(dir, name).mkdirs(); refreshFileTree(); }
    }

    private void showErr(Exception ex) {
        JOptionPane.showMessageDialog(this, ex.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
    }

    // ── Toolbar ───────────────────────────────────────────────────

    private JToolBar buildToolbar() {
        JToolBar bar = new JToolBar();
        bar.setFloatable(false);
        bar.setBackground(BG_TOOLBAR);
        bar.setBorder(new MatteBorder(0,0,1,0,BORDER_COLOR));
        bar.setPreferredSize(new Dimension(0, 46));

        JLabel logo = new JLabel("  Y++");
        logo.setFont(new Font("Consolas", Font.BOLD, 18));
        logo.setForeground(FG_KEYWORD);
        logo.setBorder(BorderFactory.createEmptyBorder(0,8,0,16));
        bar.add(logo);
        bar.addSeparator(new Dimension(1,30));
        bar.add(Box.createHorizontalStrut(6));
        bar.add(toolbarButton("New",         "Ctrl+N",       e -> newFile()));
        bar.add(Box.createHorizontalStrut(4));
        bar.add(toolbarButton("Open File",   "Ctrl+O",       e -> openFile()));
        bar.add(Box.createHorizontalStrut(4));
        bar.add(toolbarButton("Open Folder", "Ctrl+Shift+O", e -> openFolder()));
        bar.add(Box.createHorizontalStrut(4));
        bar.add(toolbarButton("Save",        "Ctrl+S",       e -> saveFile()));
        bar.add(Box.createHorizontalStrut(4));
        bar.add(toolbarButton("Save As",     "",             e -> saveFileAs()));
        bar.add(Box.createHorizontalGlue());

        JButton debugBtn = flatButton("Debug", new Color(0xC75450), Color.WHITE);
        debugBtn.setFont(new Font("Segoe UI", Font.BOLD, 13));
        debugBtn.setPreferredSize(new Dimension(80, 32));
        debugBtn.setMaximumSize(new Dimension(80, 32));
        debugBtn.setToolTipText("Run with Debug Tracing");
        debugBtn.addActionListener(e -> runCode(true));
        bar.add(debugBtn);
        bar.add(Box.createHorizontalStrut(6));

        JButton runBtn = flatButton("Run", BTN_RUN, BTN_RUN_FG);
        runBtn.setFont(new Font("Segoe UI", Font.BOLD, 13));
        runBtn.setPreferredSize(new Dimension(100, 32));
        runBtn.setMaximumSize(new Dimension(100, 32));
        runBtn.setToolTipText("Run  (F5)");
        runBtn.addActionListener(e -> runCode(false));
        bar.add(runBtn);
        bar.add(Box.createHorizontalStrut(12));
        return bar;
    }

    private JButton toolbarButton(String text, String tip, ActionListener action) {
        JButton btn = new JButton(text);
        btn.setFont(UI_FONT); btn.setForeground(FG_TEXT);
        btn.setBackground(new Color(0x3A3A3A)); btn.setOpaque(true);
        btn.setBorder(BorderFactory.createCompoundBorder(
            BorderFactory.createLineBorder(BORDER_COLOR,1),
            BorderFactory.createEmptyBorder(4,12,4,12)));
        btn.setFocusPainted(false);
        btn.setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        if (!tip.isEmpty()) btn.setToolTipText(tip);
        btn.addActionListener(action);
        btn.addMouseListener(new MouseAdapter() {
            public void mouseEntered(MouseEvent e) { btn.setBackground(new Color(0x4A4A4A)); }
            public void mouseExited(MouseEvent e)  { btn.setBackground(new Color(0x3A3A3A)); }
        });
        return btn;
    }

    // ── Status bar ────────────────────────────────────────────────

    private JPanel buildStatusBar() {
        JPanel bar = new JPanel(new BorderLayout());
        bar.setBackground(ACCENT);
        bar.setPreferredSize(new Dimension(0, 24));
        bar.setBorder(BorderFactory.createEmptyBorder(0,8,0,8));
        fileLabel.setFont(new Font("Segoe UI", Font.PLAIN, 12)); fileLabel.setForeground(Color.WHITE);
        statusLabel.setFont(new Font("Segoe UI", Font.PLAIN, 12)); statusLabel.setForeground(Color.WHITE);
        statusLabel.setHorizontalAlignment(SwingConstants.RIGHT);
        bar.add(fileLabel, BorderLayout.WEST);
        bar.add(statusLabel, BorderLayout.EAST);
        return bar;
    }

    // ── Keyboard shortcuts ────────────────────────────────────────

    private void bindKeys() {
        InputMap  im = editor.getInputMap(JComponent.WHEN_IN_FOCUSED_WINDOW);
        ActionMap am = editor.getActionMap();
        im.put(KeyStroke.getKeyStroke("F5"),           "run");
        im.put(KeyStroke.getKeyStroke("ctrl N"),       "new");
        im.put(KeyStroke.getKeyStroke("ctrl O"),       "open");
        im.put(KeyStroke.getKeyStroke("ctrl S"),       "save");
        im.put(KeyStroke.getKeyStroke("ctrl shift S"), "saveAs");
        im.put(KeyStroke.getKeyStroke("ctrl shift O"), "openFolder");
        am.put("run",        new AbstractAction() { public void actionPerformed(ActionEvent e) { runCode(false); } });
        am.put("new",        new AbstractAction() { public void actionPerformed(ActionEvent e) { newFile();    } });
        am.put("open",       new AbstractAction() { public void actionPerformed(ActionEvent e) { openFile();   } });
        am.put("save",       new AbstractAction() { public void actionPerformed(ActionEvent e) { saveFile();   } });
        am.put("saveAs",     new AbstractAction() { public void actionPerformed(ActionEvent e) { saveFileAs(); } });
        am.put("openFolder", new AbstractAction() { public void actionPerformed(ActionEvent e) { openFolder(); } });
    }

    // ── Run ───────────────────────────────────────────────────────
    private void runCode(boolean debugMode) {
        clearConsole();
        consolePrint(debugMode ? "Running in Debug Mode...\n" : "Running...\n", CONSOLE_SYS);
        String code = editor.getText();

        java.io.OutputStream liveOut = new java.io.OutputStream() {
            private final StringBuilder sb = new StringBuilder();
            @Override public void write(int b) {
                sb.append((char) b);
                if (b == '\n') flush();
            }
            @Override public void flush() {
                String s = sb.toString(); sb.setLength(0);
                if (!s.isEmpty()) SwingUtilities.invokeLater(() -> consolePrint(s, CONSOLE_OUT));
            }
        };
        java.io.OutputStream liveErr = new java.io.OutputStream() {
            private final StringBuilder sb = new StringBuilder();
            @Override public void write(int b) {
                sb.append((char) b);
                if (b == '\n') flush();
            }
            @Override public void flush() {
                String s = sb.toString(); sb.setLength(0);
                if (!s.isEmpty()) SwingUtilities.invokeLater(() -> consolePrint(s, CONSOLE_ERR));
            }
        };

        Thread runner = new Thread(() -> {
            PrintStream oldOut = System.out, oldErr = System.err;
            System.setOut(new PrintStream(liveOut, true));
            System.setErr(new PrintStream(liveErr, true));
            try {
                List<Token> tokens = new Lexer(code).tokenize();
                ASTNode.Program ast = new Parser(tokens).parse();
                Interpreter interpreter = new Interpreter();
                interpreter.debugMode = debugMode;
                interpreter.inputProvider = prompt -> {
                    System.out.print(prompt);
                    System.out.flush();
                    SwingUtilities.invokeLater(() -> {
                        inputPromptLabel.setForeground(new Color(0x23D18B));
                        inputField.setEnabled(true);
                        inputField.requestFocusInWindow();
                    });
                    try {
                        return inputQueue.take();
                    } catch (InterruptedException e) {
                        return "";
                    } finally {
                        SwingUtilities.invokeLater(() -> {
                            inputField.setEnabled(false);
                            inputPromptLabel.setForeground(new Color(0x6A9955));
                        });
                    }
                };
                interpreter.run(ast);
            } catch (YppException ex) {
                System.err.println("[Y++ Error] " + ex.getMessage());
            } catch (Exception ex) {
                System.err.println("[Internal Error] " + ex.getMessage());
            } finally {
                System.setOut(oldOut); System.setErr(oldErr);
                SwingUtilities.invokeLater(() -> consolePrint("\n--- Done ---\n", CONSOLE_SYS));
            }
        }, "ypp-runner");
        runner.setDaemon(true);
        runner.start();
    }


    // ── Console helpers ───────────────────────────────────────────

    private void consolePrint(String text, Color color) {
        StyledDocument doc = console.getStyledDocument();
        Style style = console.addStyle("tmp", null);
        StyleConstants.setForeground(style, color);
        StyleConstants.setFontFamily(style, "Consolas");
        StyleConstants.setFontSize(style, 13);
        try { doc.insertString(doc.getLength(), text, style); } catch (BadLocationException ignored) {}
        console.setCaretPosition(doc.getLength());
    }

    private void clearConsole() { console.setText(""); }

    // ── File operations ───────────────────────────────────────────

    private void newFile() {
        if (!confirmDiscardChanges()) return;
        editor.setText(""); insertTemplate();
        currentFile = null; dirty = false; updateTitle();
    }

    private void openFile() {
        if (!confirmDiscardChanges()) return;
        JFileChooser fc = styledChooser();
        if (fc.showOpenDialog(this) != JFileChooser.APPROVE_OPTION) return;
        File f = fc.getSelectedFile();
        try {
            editor.setText(Files.readString(f.toPath()));
            currentFile = f; dirty = false; updateTitle();
            SwingUtilities.invokeLater(this::rehighlight);
        } catch (IOException ex) {
            JOptionPane.showMessageDialog(this, "Could not open file:\n"+ex.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
        }
    }

    private void saveFile()   { if (currentFile==null) { saveFileAs(); return; } writeFile(currentFile); }

    private void saveFileAs() {
        JFileChooser fc = styledChooser();
        if (fc.showSaveDialog(this) != JFileChooser.APPROVE_OPTION) return;
        File f = fc.getSelectedFile();
        if (!f.getName().endsWith(".ypp")) f = new File(f.getPath()+".ypp");
        currentFile = f; writeFile(currentFile);
    }

    private void writeFile(File f) {
        try {
            Files.writeString(f.toPath(), editor.getText());
            dirty = false; updateTitle(); refreshFileTree();
        } catch (IOException ex) {
            JOptionPane.showMessageDialog(this, "Could not save file:\n"+ex.getMessage(), "Error", JOptionPane.ERROR_MESSAGE);
        }
    }

    private JFileChooser styledChooser() {
        JFileChooser fc = new JFileChooser(currentFile!=null ? currentFile.getParentFile()
                                         : projectRoot!=null ? projectRoot : new File("."));
        fc.setFileFilter(new FileNameExtensionFilter("Y++ Source Files (*.ypp)", "ypp"));
        return fc;
    }

    private boolean confirmDiscardChanges() {
        if (!dirty) return true;
        int r = JOptionPane.showConfirmDialog(this, "You have unsaved changes. Discard them?",
            "Unsaved Changes", JOptionPane.YES_NO_OPTION, JOptionPane.WARNING_MESSAGE);
        return r == JOptionPane.YES_OPTION;
    }

    private void confirmAndClose() { if (!confirmDiscardChanges()) return; dispose(); System.exit(0); }

    // ── UI helpers ────────────────────────────────────────────────

    private void updateTitle() {
        String name = currentFile!=null ? currentFile.getName() : "untitled.ypp";
        fileLabel.setText("  " + (dirty ? "● " : "") + name);
        setTitle((dirty ? "● " : "") + name + " — Y++ IDE");
    }

    private void updateStatus() {
        try {
            int pos  = editor.getCaretPosition();
            int line = editor.getDocument().getDefaultRootElement().getElementIndex(pos)+1;
            int col  = pos - editor.getDocument().getDefaultRootElement().getElement(line-1).getStartOffset()+1;
            statusLabel.setText("Ln "+line+", Col "+col+"  ");
        } catch (Exception ignored) {}
    }

    private JLabel styledLabel(String text, Font font, Color fg) {
        JLabel l = new JLabel(text); l.setFont(font); l.setForeground(fg); return l;
    }

    private JButton flatButton(String text, Color bg, Color fg) {
        JButton btn = new JButton(text);
        btn.setBackground(bg); btn.setForeground(fg); btn.setOpaque(true);
        btn.setBorderPainted(false); btn.setFocusPainted(false);
        btn.setCursor(Cursor.getPredefinedCursor(Cursor.HAND_CURSOR));
        btn.setFont(UI_BOLD);
        return btn;
    }

    private void insertTemplate() {
        editor.setText(
            "Import ycomponents *\n\n" +
            "PRINT: \"Hello, World!\";\n\n" +
            "NUM 1 {\n" +
            "    integer applecount = 3i,\n" +
            "    double appleweight = 2.4d,\n" +
            "}\n\n" +
            "STRING 1 {\n" +
            "    slong fun = \"fun\",\n" +
            "    schar dumb = \"d\",\n" +
            "}\n\n" +
            "together = (STRING 1)fun + (STRING 1)dumb;\n" +
            "PRINT: string() together;\n\n" +
            "EXCEPTION CONCAT() {\n" +
            "    maybetogether = (STRING 1)fun + (NUM 1)applecount;\n" +
            "    PRINT: stringint() maybetogether;\n" +
            "}\n");
    }

    private Image makeIcon() {
        BufferedImage img = new BufferedImage(32, 32, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        g.setColor(FG_KEYWORD); g.fillRoundRect(0, 0, 32, 32, 8, 8);
        g.setColor(Color.WHITE); g.setFont(new Font("Consolas", Font.BOLD, 14)); g.drawString("Y+", 4, 22);
        g.dispose();
        return img;
    }

    // ── Syntax Highlighter ────────────────────────────────────────

    private void rehighlight() {
        StyledDocument doc = (StyledDocument) editor.getDocument();
        String text;
        try { text = doc.getText(0, doc.getLength()); } catch (BadLocationException e) { return; }
        StyleContext sc = StyleContext.getDefaultStyleContext();
        doc.setCharacterAttributes(0, doc.getLength(),
            sc.addAttribute(SimpleAttributeSet.EMPTY, StyleConstants.Foreground, FG_TEXT), true);
        applyColor(doc, text, "\\\\\\\\.*?\\\\\\\\",                          FG_COMMENT, sc);
        applyColor(doc, text, "\"[^\"]*\"",                                   FG_STRING,  sc);
        applyColor(doc, text, "\\b(func|NEW|global|while|NOT)\\b",            FG_FUNC,    sc);
        applyColor(doc, text, "\\b(PRINT|Import|NUM|STRING|EXCEPTION|CONCAT|Public|class|Network|Server|primitivedataStream|reader|userinput)\\b", FG_KEYWORD, sc);
        applyColor(doc, text, "\\b(integer|smallint|double|slong|schar)\\b",   FG_TYPE,    sc);
        applyColor(doc, text, "\\b(int|double|string|stringint)\\(\\)",        FG_CAST,    sc);
        applyColor(doc, text, "::",                                            FG_KEYWORD, sc);
        applyColor(doc, text, "\\b[0-9]+(\\.[0-9]+)?(si|i|d)?\\b",           FG_NUMBER,  sc);
    }

    private void applyColor(StyledDocument doc, String text, String regex, Color color, StyleContext sc) {
        AttributeSet attr = sc.addAttribute(SimpleAttributeSet.EMPTY, StyleConstants.Foreground, color);
        Matcher m = Pattern.compile(regex, Pattern.DOTALL).matcher(text);
        while (m.find()) doc.setCharacterAttributes(m.start(), m.end()-m.start(), attr, false);
    }

    // ── Line number gutter ────────────────────────────────────────

    static class LineGutter extends JPanel implements DocumentListener, CaretListener {
        private final JTextPane editor;
        LineGutter(JTextPane editor) {
            this.editor = editor;
            setBackground(BG_GUTTER);
            setPreferredSize(new Dimension(48, 0));
            editor.getDocument().addDocumentListener(this);
            editor.addCaretListener(this);
        }
        @Override protected void paintComponent(Graphics g) {
            super.paintComponent(g);
            Graphics2D g2 = (Graphics2D) g;
            g2.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
            g2.setFont(EDITOR_FONT); g2.setColor(FG_GUTTER);
            Element root = editor.getDocument().getDefaultRootElement();
            FontMetrics fm = g2.getFontMetrics();
            int lh = fm.getHeight();
            try {
                Rectangle clip = g.getClipBounds();
                int s = clip!=null ? clip.y/lh : 0;
                int e = clip!=null ? Math.min((clip.y+clip.height)/lh+2, root.getElementCount()) : root.getElementCount();
                for (int i = s; i < e; i++) {
                    Rectangle r = editor.modelToView2D(root.getElement(i).getStartOffset()).getBounds();
                    String num = String.valueOf(i+1);
                    g2.drawString(num, getWidth()-fm.stringWidth(num)-6, r.y+fm.getAscent());
                }
            } catch (BadLocationException ignored) {}
        }
        public void insertUpdate(DocumentEvent e)  { repaint(); }
        public void removeUpdate(DocumentEvent e)  { repaint(); }
        public void changedUpdate(DocumentEvent e) { repaint(); }
        public void caretUpdate(CaretEvent e)      { repaint(); }
    }

    // ── Main ──────────────────────────────────────────────────────

    public static void main(String[] args) {
        try { UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName()); }
        catch (Exception ignored) {}
        SwingUtilities.invokeLater(YppIDE::new);
    }

    // ── IntelliSense / Autocomplete ───────────────────────────────

    class IntelliSense {
            {"Import",              "Import ",                  "keyword"},
            {"Import ycomponents *","Import ycomponents *\n",  "import statement"},
            {"Import ynetworking *","Import ynetworking *\n",  "import statement"},
            {"PRINT:",              "PRINT: ",                  "keyword"},
            {"PRINT: string()",     "PRINT: string() ",        "print as string"},
            {"PRINT: int()",        "PRINT: int() ",           "print as int"},
            {"PRINT: double()",     "PRINT: double() ",        "print as double"},
            {"PRINT: stringint()",  "PRINT: stringint() ",     "print string+int"},
            {"NUM",                 "NUM ",                     "numeric block"},
            {"STRING",              "STRING ",                  "string block"},
            {"EXCEPTION CONCAT()","EXCEPTION CONCAT()",        "exception concat block"},
            {"func",                "func ",                    "function declaration"},
            {"NEW",                 "NEW ",                     "instantiate function"},
            {"global",              "global ",                  "auto-executing block"},
            {"while",               "while () {\n    \n}",       "while loop"},
            {"NOT:",                "NOT: ",                    "logical NOT"},
            {"Network",             "new Network(\"127.0.0.1\", 5000)", "socket client"},
            {"Server",              "new Server(5000)",         "socket server"},
            {"primitivedataStream", "new primitivedataStream()","data stream wrapper"},
            {"reader(userinput())", "reader(userinput())",      "console input reader"},
            {"integer",             "integer ",                 "type: integer"},
            {"smallint",            "smallint ",                "type: small int"},
            {"double",              "double ",                  "type: double"},
            {"slong",               "slong ",                   "type: slong string"},
            {"schar",               "schar ",                   "type: schar string"},
            {"int()",               "int()",                   "cast to int"},
            {"double()",            "double()",                "cast to double"},
            {"string()",            "string()",                "cast to string"},
            {"stringint()",         "stringint()",             "cast to string+int"},
        };

        private final JPopupMenu    popup   = new JPopupMenu();
        private final JList<String> list    = new JList<>();
        private JTextPane           target;
        private String              prefix  = "";
        private List<String[]>      matches = new ArrayList<>();

        IntelliSense() {
            list.setBackground(new Color(0x252526));
            list.setForeground(new Color(0xD4D4D4));
            list.setFont(new Font("Consolas", Font.PLAIN, 13));
            list.setSelectionBackground(new Color(0x094771));
            list.setSelectionForeground(Color.WHITE);
            list.setFixedCellHeight(22);
            list.setBorder(BorderFactory.createEmptyBorder(2, 4, 2, 4));
            list.addMouseListener(new MouseAdapter() {
                @Override public void mouseClicked(MouseEvent e) { if (e.getClickCount()==2) applySelected(); }
            });
            JScrollPane scroll = new JScrollPane(list);
            scroll.setBorder(BorderFactory.createLineBorder(new Color(0x007ACC), 1));
            scroll.setVerticalScrollBarPolicy(ScrollPaneConstants.VERTICAL_SCROLLBAR_AS_NEEDED);
            popup.setBorder(BorderFactory.createEmptyBorder());
            popup.setBackground(new Color(0x252526));
            popup.add(scroll);
        }

        void attach(JTextPane pane) { this.target = pane; }

        void onDocumentChange() {
            SwingUtilities.invokeLater(() -> {
                prefix = currentWordPrefix();
                if (prefix.length() < 1) { hide(); return; }
                matches = computeMatches(prefix);
                if (matches.isEmpty()) { hide(); return; }
                showPopup();
            });
        }

        private String currentWordPrefix() {
            try {
                int cp = target.getCaretPosition();
                String text = target.getDocument().getText(0, cp);
                int start = cp;
                while (start > 0) {
                    char c = text.charAt(start-1);
                    if (c=='\n'||c==' '||c=='\t'||c==';'||c=='{'||c=='}') break;
                    start--;
                }
                return text.substring(start, cp);
            } catch (BadLocationException e) { return ""; }
        }

        private List<String[]> computeMatches(String pfx) {
            String lower = pfx.toLowerCase();
            List<String[]> result = new ArrayList<>();
            for (String[] c : ALL_COMPLETIONS) if (c[0].toLowerCase().startsWith(lower)) result.add(c);
            return result;
        }

        private void showPopup() {
            DefaultListModel<String> model = new DefaultListModel<>();
            for (String[] m : matches) model.addElement(m[0] + "  \u2014 " + m[2]);
            list.setModel(model); list.setSelectedIndex(0);
            int rows = Math.min(matches.size(), 9);
            list.setVisibleRowCount(rows);
            popup.setPreferredSize(new Dimension(400, rows*22+8));
            try {
                Rectangle r = target.modelToView2D(target.getCaretPosition()).getBounds();
                popup.show(target, r.x, r.y+r.height);
                target.requestFocusInWindow();
            } catch (BadLocationException ignored) {}
        }

        boolean isVisible() { return popup.isVisible(); }
        void hide() { if (popup.isVisible()) popup.setVisible(false); }

        void selectNext() {
            int i = list.getSelectedIndex();
            if (i < list.getModel().getSize()-1) list.setSelectedIndex(i+1);
            list.ensureIndexIsVisible(list.getSelectedIndex());
        }

        void selectPrev() {
            int i = list.getSelectedIndex();
            if (i > 0) list.setSelectedIndex(i-1);
            list.ensureIndexIsVisible(list.getSelectedIndex());
        }

        void applySelected() {
            int idx = list.getSelectedIndex();
            if (idx<0 || idx>=matches.size()) return;
            String insert = matches.get(idx)[1];
            try {
                int cp = target.getCaretPosition();
                int pl = prefix.length();
                target.getDocument().remove(cp-pl, pl);
                target.getDocument().insertString(target.getCaretPosition(), insert, null);
            } catch (BadLocationException ignored) {}
            hide();
        }
    }
}
