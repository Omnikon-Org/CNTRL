# Accessibility Guide

## Overview

This document describes the keyboard accessibility improvements made to the CNTRL Browser main window and explains how to verify the focus order.

## Keyboard Focus Order

The primary controls can be reached using the **Tab** key in the following order:

1. Back button
2. Forward button
3. Reload / Stop button
4. Address bar
5. Settings button
6. Browser tabs
7. Close tab buttons
8. New tab button
9. Window controls (Windows only)

## Accessibility Improvements

- Added descriptive `aria-label` attributes to navigation buttons.
- Added an `aria-label` to the address bar.
- Added an `aria-label` to the Settings button.
- Added an `aria-label` to the New Tab button.
- Added `role="tab"` and `tabindex="0"` to browser tabs.
- Added `aria-selected` to indicate the active tab.
- Added descriptive labels for tab close buttons.

## Verification Steps

1. Start the application.
2. Press the **Tab** key repeatedly.
3. Verify each primary control receives focus in a logical order.
4. Verify screen readers announce each control correctly.
5. Press **Shift + Tab** to verify reverse navigation.
6. Ensure the active tab exposes `aria-selected="true"`.

## Expected Result

- All primary controls are keyboard accessible.
- Focus order is predictable and logical.
- Screen readers announce meaningful labels.