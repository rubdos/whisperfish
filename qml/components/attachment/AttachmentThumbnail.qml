// SPDX-FileCopyrightText: 2021 Mirian Margiani
// SPDX-License-Identifier: AGPL-3.0-or-later
import QtQuick 2.6
import Sailfish.Silica 1.0
import Nemo.Thumbnailer 1.0

MouseArea {
    id: root
    property var attach: null
    property bool highlighted: containsPress
    property bool _hasAttach: attach !== null
    property bool _isAnimated: _hasAttach ? /\.(gif)$/i.test(attach.data) : false
    property bool _isVideo: _hasAttach ? /^video\//.test(attach.type) : false
    property bool _isAnimatedPaused: false

    onClicked: {
        if (!_hasAttach) {
            return
        } else if (_isAnimatedPaused && animationLoader.item) {
            _isAnimatedPaused = false
            animationLoader.item.paused = false
        } else {
            var _debugMode = SettingsBridge.boolValue("debug_mode")
            var _viewPage = '../../pages/ViewImagePage.qml'
            if (_isVideo) _viewPage = '../../pages/ViewVideoPage.qml'

            pageStack.push(_viewPage, {
                               'title': MessageModel.peerName,
                               // TODO don't show the file path once attachments work reliably (#many)
                               //      and attachments are saved in a WF-controlled directory (#253)
                               'subtitle': attach.data,
                               // when not in debug mode, it is ok to fade the file path if it is too long
                               'titleOverlay.subtitleItem.wrapMode': _debugMode ? Text.Wrap : Text.NoWrap,
                               'path': attach.data,
                               'isAnimated': _isAnimated,
                           })
        }
    }

    // TODO handle missing files and failed thumbnails
    // TODO fix: there are no thumbnails for video files in Whisperfish, even though
    //      the thumbnailer supports videos
    Thumbnail {
        visible: !_isAnimated
        width: parent.width; height: parent.height
        source: (!_isAnimated && _hasAttach) ? attach.data : ''
        sourceSize { width: width; height: height }

        onStatusChanged: {
            if (status === Thumbnail.Error && _hasAttach) {
                console.warn("thumbnail failed for", attach.data)
            }
        }
    }

    Loader {
        id: animationLoader
        anchors.fill: parent
        asynchronous: true
        sourceComponent: _isAnimated ? animatedComponent : null
    }

    HighlightImage {
        highlighted: parent.highlighted ? true : undefined
        anchors.centerIn: parent
        width: Theme.iconSizeLarge; height: width
        source: (_isVideo || _isAnimatedPaused) ? 'image://theme/icon-l-play' : ''
    }

    Rectangle {
        anchors.fill: parent
        visible: highlighted
        color: Theme.highlightBackgroundColor
        opacity: Theme.opacityFaint
    }

    Component {
        id: animatedComponent
        AnimatedImage {
            // TODO Find the most intuitive way to show a gif, restart it,
            // and show it on a separate page. Is it ok if the inline view is cropped?
            property int rounds: 0
            property int maxRounds: 2
            fillMode: Image.PreserveAspectCrop
            source: _hasAttach ? attach.data : ''
            onCurrentFrameChanged: if (currentFrame === 0) rounds++
            onRoundsChanged: {
                if (rounds <= maxRounds) return
                rounds = 0
                paused = true
                _isAnimatedPaused = true
            }
        }
    }
}
