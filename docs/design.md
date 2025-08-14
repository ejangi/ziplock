# **ZipLock Application Design**

This document outlines the comprehensive user interface (UI) and user experience (UX) design principles for all ZipLock frontend clients, including error handling, validation feedback, and interaction patterns. The goal is to create a clean, modern, and highly usable application that prioritizes clarity and simplicity.

## **1\. Design Philosophy**

The application will adopt a **flat design** philosophy. This approach is characterized by minimalist aesthetics, the avoidance of gradients and drop shadows, and a focus on simplicity. The design will be simple to use and easy to understand, with a generous use of **white space** to make information digestible and reduce visual clutter.

* **Simplicity:** Every element on the screen should serve a clear purpose. Unnecessary embellishments are to be avoided.
* **Clarity:** Information, especially text and icons, must be easily readable and understandable at a glance.
* **Responsiveness:** The layout should adapt gracefully to different screen sizes, from mobile phones to large desktop monitors, without compromising usability.

## **2\. Visual Elements**

### **2.1 Color Palette**

The color palette will be minimalist and modern, using a light or dark theme with a single accent color.

* **Primary Background:** A neutral color for the main interface, such as a soft white (\#F8F9FA) for the light theme or a dark gray (\#212529) for the dark theme.
* **Secondary Background:** A slightly darker or lighter shade for component backgrounds, like \#E9ECEF or \#343A40.
* **Text:** A high-contrast color for readability. \#212529 for light mode and \#F8F9FA for dark mode.
* **Accent Color:** The primary brand color will be **\#8338ec**, a vibrant purple, used for interactive elements (buttons, links) and highlights. Lighter and darker shades of this color, along with shades of black and white, will be used throughout the UI to create a cohesive palette.

#### **2.1.1 Validation Colors**

For form validation and user feedback, specific colors are used to indicate success and error states:

* **Success/Valid Color:** **\#06d6a0** - A vibrant green used to indicate valid input, successful operations, or positive feedback. This color is used for:
  - Valid passphrase fields that meet all security requirements
  - Matching confirmation fields
  - Successful operation indicators

* **Error/Invalid Color:** **\#ef476f** - A vibrant red/pink used to indicate invalid input, errors, or negative feedback. This color is used for:
  - Invalid or weak passphrase fields
  - Mismatched confirmation fields
  - Error messages and warnings
  - Failed operation indicators

### **2.2 Typography**

Typography is a key element for readability.

* **Font Family:** A modern, sans-serif font will be used, such as **Inter**. This font is highly legible and works well at various sizes.
* **Font Size:** Fonts should be large enough to be easily read, especially on mobile devices. A clear hierarchy of font sizes will be established for titles, subtitles, and body text.
* **Weight:** Use of font weights (e.g., regular, semibold, bold) will be limited and used purposefully to differentiate headings and important information.

### **2.3 Icons**

Icons will be simple, flat, and modern.

* **Consistency:** The style of all icons must be consistent across the application.
* **Clarity:** Icons should be immediately recognizable and their function should be obvious.
* **Source:** The application uses **[Iconoir](https://iconoir.com/)** - a beautiful collection of free SVG icons that maintain a consistent style and ensure scalability across all platforms.

### **2.4 Playful Elements**

To reinforce the application's brand and functionality, playful elements can be incorporated using the motifs of **padlocks**, **zips**, and **keys**. These could be used in loading animations, transition effects, or as visual feedback (e.g., a "padlock" icon closing when the app locks, or a "zip" animation for a data save).

## **3\. Core Layouts and Components**

The frontend will be composed of a few key views and reusable components.

### **3.1 Main Dashboard (Credential List)**

This is the primary view users will see after unlocking the app.

* A list of credentials, with each item clearly displaying the credential's title and a brief summary.
* A search bar at the top with a magnifying glass icon for easy access.
* A sidebar or menu for managing tags, credential types, and settings.
* A button for adding new credentials, prominently placed in an easy-to-reach location.

### **3.2 Credential Detail View**

This view displays all the information for a single credential.

* The credential's title and type are displayed at the top.
* Fields are presented in a clean, vertical list.
* Fields containing sensitive data (e.g., passwords, TOTP keys) should be masked by default and require an explicit action (e.g., a "show" button or an eye icon) to reveal the content.
* Buttons for editing, deleting, or copying field content will be clearly visible and accessible.

### **3.3 Initial Setup**

The first-run experience must be simple and guide the user through the process of creating their encrypted database.

* A step-by-step process with clear instructions.
* A strong master key validation that provides feedback on the password's strength.
* A file picker dialog to choose the location for the ziplock.7z file.

## **4. Error Display and Feedback System**

The application implements a comprehensive error display system to handle and present messages to users in a clear and consistent manner across all views.

### **4.1 Alert Component System**

The error display system provides a unified way to show error messages, warnings, success notifications, and informational alerts.

#### **Alert Types and Styling**
- **Error Alerts:** Use `ERROR_RED` (#ef476f) for critical issues requiring user attention
- **Warning Alerts:** Use `WARNING_YELLOW` (#fcbf49) for cautionary messages
- **Success Alerts:** Use `SUCCESS_GREEN` (#06d6a0) for positive feedback and confirmations
- **Info Alerts:** Use neutral colors for informational messages

#### **Alert Components**
- Reusable alert components with consistent styling across all views
- Dismissible alerts with optional close buttons
- Proper integration with the theme system and color palette
- Support for both title and message content

### **4.2 User-Friendly Error Messages**

The system automatically converts technical backend error messages to user-readable text:

| Backend Error | User-Friendly Message |
|---------------|----------------------|
| "Failed to bind to socket" | "Unable to start the backend service. Please check if another instance is running." |
| "Authentication failed" | "Incorrect passphrase. Please check your password and try again." |
| "Archive not found" | "The password archive file could not be found. Please check the file path." |
| "Permission denied" | "Permission denied. Please check file permissions or run with appropriate privileges." |
| "Connection lost" | "Lost connection to the backend service. Please restart the application." |

### **4.3 Cross-View Integration**

- Centralized toast notification system for all user messages (see Section 9)
- Global toast manager handles error, warning, success, and info notifications
- Views send messages to parent application for unified toast display
- Consistent error display across wizard, main interface, and other views

### **4.4 Design Principles for Error Display**

#### **Consistency**
- All error displays use the same visual style and color usage
- Uniform spacing and typography following the established design language
- Consistent placement at the top of content areas

#### **Clarity**
- Error messages written in plain language
- Technical details abstracted away from users
- Clear distinction between different types of issues through color coding

#### **Non-Intrusive**
- Toast notifications appear as overlays without blocking the interface
- Auto-dismissing toasts (5-second default) reduce UI clutter
- Dismissible design allows users to continue working
- Loading states provide feedback without interrupting workflow

#### **Accessibility**
- High contrast colors for readability
- Clear visual hierarchy with icons and typography
- Keyboard navigation support through standard UI components

## **5. Password Input and Validation Design**

### **5.1 Password Visibility Toggle**

Password fields include a professional eye icon toggle for showing/hiding password content.

#### **Design Rationale**
- **Visual Consistency:** Professional appearance matching modern password managers
- **Space Efficiency:** Icons take less horizontal space than text labels
- **Universal Recognition:** Eye symbol is universally understood across cultures
- **Design Cohesion:** SVG icons match the overall flat design aesthetic
- **Scalability:** Vector icons work at any size and resolution

#### **Implementation**
- Centralized component in theme system for consistency
- Used across all password input fields (wizard, open repository, etc.)
- Embedded SVG ensures no external dependencies
- Consistent styling with secondary button styles
- Optimal sizing (16x16px) for visibility

### **5.2 Real-Time Validation Feedback**

Password strength and validation provide immediate user feedback:

#### **Strength Levels**
| Level | Score Range | Color | Visual Treatment |
|-------|-------------|-------|------------------|
| Very Weak | 0-20 | Red (#ef476f) | Unacceptable - prevent submission |
| Weak | 21-40 | Red (#ef476f) | Unacceptable - prevent submission |
| Fair | 41-60 | Yellow (#fcbf49) | Borderline - show warnings |
| Good | 61-80 | Green (#06d6a0) | Acceptable - allow submission |
| Strong | 81-95 | Green (#06d6a0) | Acceptable - positive feedback |
| Very Strong | 96-100 | Purple (#8338ec) | Excellent - highlight achievement |

#### **Validation Display**
- Real-time strength assessment as user types
- Color-coded strength indicators using theme colors
- List of requirement violations with clear descriptions
- List of satisfied requirements for positive reinforcement
- Submit button state tied to validation results

## **6. Navigation and State Management Design**

### **6.1 Modal and Dialog Patterns**

For modal-like experiences (wizard, open repository), consistent navigation patterns:

#### **Cancel Behavior**
- Cancel buttons return users to the previous logical state
- Clear visual feedback for cancellation actions
- Proper state cleanup to prevent user confusion
- Consistent with user expectations across the application

#### **State Transitions**
- Clear visual indicators for different states (input, loading, success, error)
- Smooth transitions between states without jarring changes
- Loading states with appropriate feedback
- Error recovery options where applicable

### **6.2 File Selection Interface**

#### **File Dialog Integration**
- Native file dialog integration for platform consistency
- Appropriate file type filtering (.7z files)
- Clear visual feedback showing selected file names
- Graceful fallback for systems without native dialogs

#### **Path Display**
- Clear display of selected file paths
- Truncation handling for long paths
- Visual distinction between selected and placeholder states

## **7. Icon System and Visual Language**

### **7.1 Icon Guidelines**

Building on the [Iconoir icon system](https://github.com/iconoir-icons/iconoir/tree/main/icons) with additional considerations:

#### **Custom Icons**
- **Eye Icon:** Password visibility toggle using embedded SVG
- **Padlock Motifs:** Reinforcing brand identity in loading and transition states
- **Zip Motifs:** Visual callbacks to the underlying storage format
- **Key Motifs:** Security and authentication visual cues

#### **Icon Usage Principles**
- **Functional Icons:** Every icon must serve a clear functional purpose
- **Recognition:** Icons should be immediately recognizable
- **Consistency:** Maintain consistent style across all icons
- **Accessibility:** Icons paired with appropriate labels or tooltips where needed

### **7.2 Playful Elements**

Incorporate brand-appropriate playful elements:

- **Loading Animations:** Padlock closing/opening, zip closing animations
- **Transition Effects:** Subtle key/lock metaphors for state changes
- **Success Feedback:** Visual confirmation using security metaphors
- **Empty States:** Friendly illustrations using lock/key/zip themes

## **8. Responsive Design Considerations**

### **8.1 Layout Adaptation**

- **Mobile-First Approach:** Design scales down gracefully
- **Touch Targets:** Appropriate sizing for touch interfaces
- **Content Reflow:** Layouts adapt to different screen proportions
- **Navigation Adaptation:** Menu and navigation patterns suitable for various screen sizes

### **8.2 Density Variations**

- **High DPI Support:** Vector icons and scalable elements
- **Text Scaling:** Respect system text size preferences
- **Spacing Adaptation:** Maintain proportional spacing across densities

## **9. Toast Notification System**

The application implements a centralized toast notification system that provides non-intrusive user feedback across all views.

### **9.1 Toast Architecture**

#### **Core Components**
- **Toast Manager:** Centralized management of multiple notifications with auto-dismiss and positioning
- **Toast Types:** Error, Warning, Success, and Info notifications using existing theme colors
- **Positioning:** Configurable placement (default: bottom-right corner)
- **Auto-Dismiss:** 5-second default duration with manual dismiss option

#### **Design Integration**
- **Error Toasts:** Use `ERROR_RED` (#ef476f) with error icon
- **Warning Toasts:** Use `WARNING_YELLOW` (#fcbf49) with warning icon
- **Success Toasts:** Use `SUCCESS_GREEN` (#06d6a0) with check icon
- **Info Toasts:** Use `LOGO_PURPLE` (#8338ec) with alert icon

#### **User Experience**
- **Non-Intrusive:** Appears in bottom-right corner without disrupting main interface layout
- **Auto-Cleanup:** Maximum 3 visible toasts with automatic expiration
- **Consistent Styling:** Reuses existing alert container styles and iconography
- **Accessibility:** High contrast colors, dismissible design, keyboard navigation support

### **9.2 Implementation Benefits**

#### **Centralized Messaging**
All user-facing messages are managed through a single toast system, ensuring:
- Consistent appearance and behavior across views
- Unified message handling and routing
- Simplified view code without local error state management

#### **Migration from Inline Alerts**
The toast system replaces previous inline alert implementations:
- **Before:** Each view managed `current_error: Option<AlertMessage>` with layout-disrupting inline alerts
- **After:** Views send messages to parent application for centralized toast display
- **Result:** Cleaner view code and better user experience

## **10. Future Design Enhancements**

### **10.1 Advanced Error Handling**

Potential improvements to the error and feedback systems:

- **Error Categories:** Grouping related errors with contextual help
- **Error Recovery Actions:** Specific user actions suggested for resolution
- **Progressive Disclosure:** Advanced error details available on demand
- **Enhanced Toast Features:** Rich content support, priority queuing, persistent storage

### **10.2 Enhanced Interactions**

- **Keyboard Shortcuts:** Comprehensive keyboard navigation support
- **Gesture Support:** Touch and trackpad gesture integration where appropriate
- **Animation Framework:** Consistent motion design language
- **Theme Extensions:** Support for user customization while maintaining brand consistency

This comprehensive design framework ensures a consistent, accessible, and delightful user experience across all ZipLock frontend applications while maintaining the clean, modern aesthetic that defines the application's visual identity.
