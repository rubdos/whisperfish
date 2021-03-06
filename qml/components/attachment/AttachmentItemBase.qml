// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nemo.Thumbnailer 1.0

MouseArea {
    id: root
    property var attach: null
    property bool highlighted: containsPress
    property string icon: ''
    property bool enableDefaultClickAction: true
    default property alias contents: attachmentContentItem.data

    // check _effectiveEnableClick in derived types, not enableDefaultClickAction
    property bool _effectiveEnableClick: _hasAttach && enableDefaultClickAction
    property bool _hasAttach: attach !== null

    function mimeToIcon(mimeType) {
        if (root.icon !== '') return root.icon
        var icon = Theme.iconForMimeType(mimeType)
        return icon === "image://theme/icon-m-file-other" ? "image://theme/icon-m-attach" : icon
    }

    function lastPartOfPath(path) {
        path = path.replace(/\/+/g, '/');
        if (path === "/") return "";
        var i = path.lastIndexOf("/");
        if (i < -1) return path;
        return path.substring(i+1);
    }

    Row {
        anchors.fill: parent
        spacing: Theme.paddingMedium
        Item {
            height: parent.height; width: height
            clip: true
            Rectangle {
                anchors { fill: parent; margins: -parent.width/2 }
                rotation: 45
                gradient: Gradient {
                    GradientStop { position: 0.0; color: "transparent" }
                    GradientStop { position: 0.4; color: "transparent" }
                    GradientStop { position: 1.0; color: Theme.rgba(Theme.secondaryColor, 0.1) }
                }
            }

            Thumbnail {
                id: thumb
                anchors.fill: parent
                source: (icon === '' && _hasAttach) ? attach.data : ''
                sourceSize { width: width; height: height }
            }
            HighlightImage {
                anchors.centerIn: parent
                highlighted: root.highlighted ? true : undefined
                width: Theme.iconSizeMedium; height: width
                visible: thumb.status === Thumbnail.Error ||
                         thumb.status === Thumbnail.Null
                source: _hasAttach ? mimeToIcon(attach.type) : ''
            }
        }

        Item {
            id: attachmentContentItem
            width: parent.width - parent.height /* icon width */ - parent.spacing
            height: parent.height

            /* children... */
        }
    }

    Rectangle {
        anchors.fill: parent
        visible: highlighted
        color: Theme.highlightBackgroundColor
        opacity: Theme.opacityFaint
    }
}
