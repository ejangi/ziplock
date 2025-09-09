package com.ziplock.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ziplock.ffi.ZipLockNative
import com.ziplock.ffi.ZipLockNativeHelper
import com.ziplock.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CredentialTemplateSelectionScreen(
    onTemplateSelected: (ZipLockNativeHelper.CredentialTemplate) -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier
) {
    // Get all available templates
    val templates = remember { ZipLockNativeHelper.getAvailableTemplates() }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(ZipLockColors.LightBackground)
    ) {
        // Header
        TemplateSelectionHeader(
            onCancel = onCancel,
            modifier = Modifier.fillMaxWidth()
        )

        // Title and subtitle
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = ZipLockSpacing.Standard)
        ) {
            Text(
                text = "Choose Credential Type",
                style = ZipLockTypography.Header,
                color = ZipLockColors.DarkText,
                fontWeight = FontWeight.SemiBold
            )

            Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

            Text(
                text = "Select the type of credential you want to create. Each type has predefined fields to help you organize your information.",
                style = ZipLockTypography.Normal,
                color = ZipLockColors.LightGrayText,
                textAlign = TextAlign.Start
            )
        }

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        // Templates list
        LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(
                horizontal = ZipLockSpacing.Standard,
                vertical = ZipLockSpacing.Small
            ),
            verticalArrangement = Arrangement.spacedBy(ZipLockSpacing.Small)
        ) {
            items(templates) { template ->
                TemplateListItem(
                    template = template,
                    onClick = { onTemplateSelected(template) },
                    modifier = Modifier.fillMaxWidth()
                )
            }

            // Add bottom padding for better scrolling experience
            item {
                Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun TemplateSelectionHeader(
    onCancel: () -> Unit,
    modifier: Modifier = Modifier
) {
    Surface(
        modifier = modifier,
        color = ZipLockColors.White,
        shadowElevation = 2.dp
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(ZipLockSpacing.Standard),
            horizontalArrangement = Arrangement.SpaceBetween,
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text(
                text = "New Credential",
                style = ZipLockTypography.Header,
                color = ZipLockColors.DarkText,
                fontWeight = FontWeight.SemiBold
            )

            IconButton(
                onClick = onCancel,
                modifier = Modifier
                    .size(32.dp)
                    .clip(CircleShape)
                    .background(ZipLockColors.VeryLightGray)
            ) {
                Icon(
                    imageVector = ZipLockIcons.Close,
                    contentDescription = "Cancel",
                    tint = ZipLockColors.LightGrayText,
                    modifier = Modifier.size(18.dp)
                )
            }
        }
    }
}

@Composable
private fun TemplateListItem(
    template: ZipLockNativeHelper.CredentialTemplate,
    onClick: () -> Unit,
    modifier: Modifier = Modifier
) {
    Card(
        modifier = modifier
            .clickable { onClick() },
        colors = CardDefaults.cardColors(containerColor = ZipLockColors.White),
        elevation = CardDefaults.cardElevation(defaultElevation = 1.dp),
        shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(ZipLockSpacing.Standard),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // Template type icon
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(ZipLockColors.LogoPurple.copy(alpha = 0.1f)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = getCredentialTypeIcon(template.name),
                    contentDescription = "Template type: ${template.name}",
                    tint = ZipLockColors.LogoPurple,
                    modifier = Modifier.size(24.dp)
                )
            }

            Spacer(modifier = Modifier.width(ZipLockSpacing.Standard))

            // Template details
            Column(
                modifier = Modifier.weight(1f)
            ) {
                // Template name
                Text(
                    text = formatTemplateName(template.name),
                    style = ZipLockTypography.Medium,
                    color = ZipLockColors.DarkText,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )

                Spacer(modifier = Modifier.height(4.dp))

                // Template description
                Text(
                    text = template.description,
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.LightGrayText,
                    maxLines = 2,
                    overflow = TextOverflow.Ellipsis
                )

                // Field count info
                if (template.fields.isNotEmpty()) {
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = "${template.fields.size} fields",
                        style = ZipLockTypography.Small,
                        color = ZipLockColors.LogoPurple,
                        modifier = Modifier
                            .background(
                                ZipLockColors.LogoPurple.copy(alpha = 0.1f),
                                RoundedCornerShape(4.dp)
                            )
                            .padding(horizontal = 6.dp, vertical = 2.dp)
                    )
                }
            }

            // Arrow indicator
            Icon(
                imageVector = ZipLockIcons.ArrowRight,
                contentDescription = "Select template",
                tint = ZipLockColors.LightGrayText,
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

/**
 * Format template name for display
 */
private fun formatTemplateName(name: String): String {
    return name.split("_")
        .joinToString(" ") { word ->
            word.replaceFirstChar {
                if (it.isLowerCase()) it.titlecase() else it.toString()
            }
        }
}
