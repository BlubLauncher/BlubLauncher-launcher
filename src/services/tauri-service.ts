import { openUrl } from '@tauri-apps/plugin-opener';
import { invoke } from '@tauri-apps/api/core';
import type { ImagePreviewPayload, ImagePreviewResponse } from '../types/fileSystem';

/**
 * Opens a given URL in the default external application (usually the web browser).
 * Requires the `plugin-opener` to be configured and appropriate permissions granted.
 * See: https://v2.tauri.app/plugin/opener/
 *
 * @param url The URL to open (e.g., 'https://example.com').
 * @throws If the opener plugin fails to open the URL (e.g., due to permissions or invalid URL).
 */
export const openExternalUrl = (url: string): Promise<void> => {
  // Directly call the plugin function. Errors propagate to the caller.
  return openUrl(url);
};

/**
 * Invokes the backend command to get a processed image preview.
 *
 * @param payload The necessary information to generate the preview (path, optional dimensions/quality).
 * @returns A promise that resolves with the image preview data.
 * @throws If the backend command fails.
 */
export const getImagePreview = (payload: ImagePreviewPayload): Promise<ImagePreviewResponse> => {
  return invoke<ImagePreviewResponse>('get_image_preview', { payload });
}; 