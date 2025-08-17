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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.ziplock.ffi.Credential
import com.ziplock.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CredentialsListScreen(
    credentials: List<Credential>,
    searchQuery: String,
    onSearchQueryChange: (String) -> Unit,
    onCredentialClick: (Credential) -> Unit,
    onCloseArchive: () -> Unit,
    onAddCredential: () -> Unit,
    onLoadMockData: (() -> Unit)? = null,
    isLoading: Boolean = false,
    errorMessage: String? = null,
    modifier: Modifier = Modifier
) {
    // Filter credentials based on search query
    val filteredCredentials = remember(credentials, searchQuery) {
        if (searchQuery.isBlank()) {
            credentials
        } else {
            credentials.filter { credential ->
                credential.title.contains(searchQuery, ignoreCase = true) ||
                credential.credentialType.contains(searchQuery, ignoreCase = true) ||
                credential.username.contains(searchQuery, ignoreCase = true) ||
                credential.url.contains(searchQuery, ignoreCase = true) ||
                credential.tags.any { tag -> tag.contains(searchQuery, ignoreCase = true) }
            }
        }
    }

    Column(
        modifier = modifier
            .fillMaxSize()
            .background(ZipLockColors.LightBackground)
    ) {
        // Header with close button
        CredentialsListHeader(
            onCloseArchive = onCloseArchive,
            onLoadMockData = onLoadMockData,
            modifier = Modifier.fillMaxWidth()
        )

        // Search bar
        CredentialsSearchBar(
            searchQuery = searchQuery,
            onSearchQueryChange = onSearchQueryChange,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = ZipLockSpacing.Standard)
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        // Error message
        errorMessage?.let { error ->
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = ZipLockSpacing.Standard),
                colors = CardDefaults.cardColors(containerColor = ZipLockColors.ErrorRed.copy(alpha = 0.1f)),
                shape = RoundedCornerShape(ZipLockSpacing.BorderRadius)
            ) {
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(ZipLockSpacing.Standard),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Icon(
                        imageVector = ZipLockIcons.ErrorCircle,
                        contentDescription = "Error",
                        tint = ZipLockColors.ErrorRed,
                        modifier = Modifier.size(20.dp)
                    )
                    Spacer(modifier = Modifier.width(ZipLockSpacing.Small))
                    Text(
                        text = error,
                        style = ZipLockTypography.Normal,
                        color = ZipLockColors.ErrorRed
                    )
                }
            }
            Spacer(modifier = Modifier.height(ZipLockSpacing.Small))
        }

        // Credentials list
        when {
            isLoading -> {
                CredentialsLoadingState(
                    modifier = Modifier.fillMaxSize()
                )
            }
            filteredCredentials.isEmpty() && searchQuery.isNotBlank() -> {
                CredentialsEmptySearchState(
                    searchQuery = searchQuery,
                    modifier = Modifier.fillMaxSize()
                )
            }
            credentials.isEmpty() -> {
                CredentialsEmptyState(
                    onAddCredential = onAddCredential,
                    modifier = Modifier.fillMaxSize()
                )
            }
            else -> {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(
                        horizontal = ZipLockSpacing.Standard,
                        vertical = ZipLockSpacing.Small
                    ),
                    verticalArrangement = Arrangement.spacedBy(ZipLockSpacing.Small)
                ) {
                    items(filteredCredentials) { credential ->
                        CredentialListItem(
                            credential = credential,
                            onClick = { onCredentialClick(credential) },
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
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CredentialsListHeader(
    onCloseArchive: () -> Unit,
    onLoadMockData: (() -> Unit)? = null,
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
                text = "Credentials",
                style = ZipLockTypography.Header,
                color = ZipLockColors.DarkText,
                fontWeight = FontWeight.SemiBold
            )

            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically
            ) {
                // Development toggle for mock data (only show if callback provided)
                onLoadMockData?.let { loadMock ->
                    IconButton(
                        onClick = loadMock,
                        modifier = Modifier
                            .size(32.dp)
                            .clip(CircleShape)
                            .background(ZipLockColors.LogoPurple.copy(alpha = 0.1f))
                    ) {
                        Icon(
                            imageVector = ZipLockIcons.Refresh,
                            contentDescription = "Load Mock Data",
                            tint = ZipLockColors.LogoPurple,
                            modifier = Modifier.size(18.dp)
                        )
                    }
                }

                // Close archive button
                IconButton(
                    onClick = onCloseArchive,
                    modifier = Modifier
                        .size(32.dp)
                        .clip(CircleShape)
                        .background(ZipLockColors.VeryLightGray)
                ) {
                    Icon(
                        imageVector = ZipLockIcons.Close,
                        contentDescription = "Close Archive",
                        tint = ZipLockColors.LightGrayText,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun CredentialsSearchBar(
    searchQuery: String,
    onSearchQueryChange: (String) -> Unit,
    modifier: Modifier = Modifier
) {
    OutlinedTextField(
        value = searchQuery,
        onValueChange = onSearchQueryChange,
        placeholder = {
            Text(
                text = "Search credentials...",
                style = ZipLockTypography.Normal,
                color = ZipLockColors.LightGrayText
            )
        },
        leadingIcon = {
            Icon(
                imageVector = ZipLockIcons.Search,
                contentDescription = "Search",
                tint = ZipLockColors.LightGrayText,
                modifier = Modifier.size(20.dp)
            )
        },
        trailingIcon = {
            if (searchQuery.isNotBlank()) {
                IconButton(
                    onClick = { onSearchQueryChange("") }
                ) {
                    Icon(
                        imageVector = ZipLockIcons.Close,
                        contentDescription = "Clear search",
                        tint = ZipLockColors.LightGrayText,
                        modifier = Modifier.size(18.dp)
                    )
                }
            }
        },
        singleLine = true,
        colors = OutlinedTextFieldDefaults.colors(
            focusedBorderColor = ZipLockColors.LogoPurple,
            unfocusedBorderColor = ZipLockColors.VeryLightGray,
            focusedTextColor = ZipLockColors.DarkText,
            unfocusedTextColor = ZipLockColors.DarkText,
            cursorColor = ZipLockColors.LogoPurple
        ),
        shape = RoundedCornerShape(ZipLockSpacing.BorderRadius),
        modifier = modifier
    )
}

@Composable
private fun CredentialListItem(
    credential: Credential,
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
            // Credential type icon
            Box(
                modifier = Modifier
                    .size(48.dp)
                    .clip(CircleShape)
                    .background(ZipLockColors.LogoPurple.copy(alpha = 0.1f)),
                contentAlignment = Alignment.Center
            ) {
                Icon(
                    imageVector = getCredentialTypeIcon(credential.credentialType),
                    contentDescription = "Credential type: ${credential.credentialType}",
                    tint = ZipLockColors.LogoPurple,
                    modifier = Modifier.size(24.dp)
                )
            }

            Spacer(modifier = Modifier.width(ZipLockSpacing.Standard))

            // Credential details
            Column(
                modifier = Modifier.weight(1f)
            ) {
                // Title
                Text(
                    text = credential.title,
                    style = ZipLockTypography.Medium,
                    color = ZipLockColors.DarkText,
                    fontWeight = FontWeight.Medium,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )

                Spacer(modifier = Modifier.height(4.dp))

                // Subtitle (username, URL, or credential type)
                val subtitle = when {
                    credential.username.isNotBlank() -> credential.username
                    credential.url.isNotBlank() -> credential.url
                    else -> credential.credentialType.replaceFirstChar { it.uppercase() }
                }

                Text(
                    text = subtitle,
                    style = ZipLockTypography.Small,
                    color = ZipLockColors.LightGrayText,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis
                )

                // Tags (if any)
                if (credential.tags.isNotEmpty()) {
                    Spacer(modifier = Modifier.height(4.dp))
                    Row(
                        horizontalArrangement = Arrangement.spacedBy(4.dp)
                    ) {
                        credential.tags.take(2).forEach { tag ->
                            Text(
                                text = "#$tag",
                                style = ZipLockTypography.Small.copy(fontSize = 10.sp),
                                color = ZipLockColors.LogoPurple,
                                modifier = Modifier
                                    .background(
                                        ZipLockColors.LogoPurple.copy(alpha = 0.1f),
                                        RoundedCornerShape(4.dp)
                                    )
                                    .padding(horizontal = 6.dp, vertical = 2.dp)
                            )
                        }
                        if (credential.tags.size > 2) {
                            Text(
                                text = "+${credential.tags.size - 2}",
                                style = ZipLockTypography.Small.copy(fontSize = 10.sp),
                                color = ZipLockColors.LightGrayText
                            )
                        }
                    }
                }
            }

            // Arrow indicator
            Icon(
                imageVector = ZipLockIcons.ArrowRight,
                contentDescription = "View credential",
                tint = ZipLockColors.LightGrayText,
                modifier = Modifier.size(20.dp)
            )
        }
    }
}

@Composable
private fun CredentialsLoadingState(
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier,
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        CircularProgressIndicator(
            color = ZipLockColors.LogoPurple,
            modifier = Modifier.size(48.dp)
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "Loading credentials...",
            style = ZipLockTypography.Medium,
            color = ZipLockColors.LightGrayText
        )
    }
}

@Composable
private fun CredentialsEmptyState(
    onAddCredential: () -> Unit,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(ZipLockSpacing.ExtraLarge),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "🔒",
            style = ZipLockTypography.ExtraLarge.copy(fontSize = 64.sp)
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "Create your first credential:",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText,
            fontWeight = FontWeight.Medium
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "Your archive is ready. Start by adding your first credential to get organized!",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText,
            textAlign = androidx.compose.ui.text.style.TextAlign.Center
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.ExtraLarge))

        ZipLockButton(
            text = "Add a Credential",
            onClick = onAddCredential,
            style = ZipLockButtonStyle.Primary,
            icon = ZipLockIcons.Plus,
            modifier = Modifier.fillMaxWidth(0.8f)
        )
    }
}

@Composable
private fun CredentialsEmptySearchState(
    searchQuery: String,
    modifier: Modifier = Modifier
) {
    Column(
        modifier = modifier.padding(ZipLockSpacing.ExtraLarge),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center
    ) {
        Text(
            text = "🔍",
            style = ZipLockTypography.ExtraLarge.copy(fontSize = 64.sp)
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Standard))

        Text(
            text = "No Results Found",
            style = ZipLockTypography.Header,
            color = ZipLockColors.DarkText,
            fontWeight = FontWeight.Medium
        )

        Spacer(modifier = Modifier.height(ZipLockSpacing.Small))

        Text(
            text = "No credentials match \"$searchQuery\". Try a different search term or check your spelling.",
            style = ZipLockTypography.Normal,
            color = ZipLockColors.LightGrayText,
            textAlign = androidx.compose.ui.text.style.TextAlign.Center
        )
    }
}
