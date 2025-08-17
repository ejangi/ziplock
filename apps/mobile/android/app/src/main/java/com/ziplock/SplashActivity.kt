package com.ziplock

import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlinx.coroutines.delay

class SplashActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        setContent {
            MaterialTheme {
                SplashScreen {
                    // Navigate to MainActivity after splash duration
                    navigateToMain()
                }
            }
        }
    }

    private fun navigateToMain() {
        val intent = Intent(this, MainActivity::class.java)
        startActivity(intent)
        finish()
    }
}

@Composable
fun SplashScreen(onSplashComplete: () -> Unit) {
    // Auto-navigate after 2.5 seconds
    LaunchedEffect(Unit) {
        delay(2500)
        onSplashComplete()
    }

    Box(
        modifier = Modifier
            .fillMaxSize()
            .background(Color.White),
        contentAlignment = Alignment.Center
    ) {
        Column(
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.Center
        ) {
            // ZipLock Logo
            Image(
                painter = painterResource(id = R.drawable.ziplock_logo),
                contentDescription = stringResource(R.string.app_name),
                modifier = Modifier
                    .size(120.dp)
                    .padding(bottom = 24.dp)
            )

            // App Title
            Text(
                text = stringResource(R.string.splash_title),
                fontSize = 32.sp,
                fontWeight = FontWeight.Bold,
                color = Color(0xFF8338EC), // logo_purple
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            // App Subtitle
            Text(
                text = stringResource(R.string.splash_subtitle),
                fontSize = 16.sp,
                fontWeight = FontWeight.Normal,
                color = Color(0xFF212529), // text_primary_light
                textAlign = TextAlign.Center,
                modifier = Modifier.padding(bottom = 32.dp)
            )

            // Loading indicator (simple text)
            Text(
                text = stringResource(R.string.loading),
                fontSize = 14.sp,
                fontWeight = FontWeight.Normal,
                color = Color(0xFF6C757D), // text_secondary_light
                textAlign = TextAlign.Center
            )
        }
    }
}



@Preview(showBackground = true)
@Composable
fun SplashScreenPreview() {
    MaterialTheme {
        SplashScreen {}
    }
}
