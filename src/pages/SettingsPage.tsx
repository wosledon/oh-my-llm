import { useEffect } from 'react';
import Typography from '@mui/material/Typography';
import TextField from '@mui/material/TextField';
import Switch from '@mui/material/Switch';
import FormControlLabel from '@mui/material/FormControlLabel';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import CircularProgress from '@mui/material/CircularProgress';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import { useProxyStore } from '../stores/proxyStore';
import { useI18n } from '../i18n';

export default function SettingsPage() {
  const { t, lang, setLang } = useI18n();
  const { config, loading, error, fetchConfig, updateConfig } = useProxyStore();

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  if (!config) return <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />;

  return (
    <Box>
      <Typography variant="h4" sx={{ fontWeight: 600, mb: 3 }}>
        {t.settings.title}
      </Typography>

      {loading && <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>{t.common.error}: {error}</Alert>
      )}

      <Box sx={{ mb: 3 }}>
        <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>{t.settings.title}</Typography>
        <Card variant="outlined" sx={{ maxWidth: 560, borderRadius: 2 }}>
          <CardContent sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            <TextField
              label={t.settings.port}
              type="number"
              value={config.port}
              onChange={(e) => updateConfig({ ...config, port: parseInt(e.target.value) || 11888 })}
              fullWidth
            />

            <TextField
              label={t.settings.shadowModelName}
              value={config.shadow_model_name || ''}
              onChange={(e) => updateConfig({ ...config, shadow_model_name: e.target.value })}
              placeholder="gpt-4"
              helperText={t.settings.shadowModelDesc}
              fullWidth
            />

            <FormControlLabel
              control={
                <Switch
                  checked={config.openai_enabled}
                  onChange={(e) => updateConfig({ ...config, openai_enabled: e.target.checked })}
                />
              }
              label={t.settings.openaiEnabled}
            />

            <FormControlLabel
              control={
                <Switch
                  checked={config.anthropic_enabled}
                  onChange={(e) => updateConfig({ ...config, anthropic_enabled: e.target.checked })}
                />
              }
              label={t.settings.anthropicEnabled}
            />

            <FormControlLabel
              control={
                <Switch
                  checked={config.auto_start}
                  onChange={(e) => updateConfig({ ...config, auto_start: e.target.checked })}
                />
              }
              label={t.settings.autoStart}
            />

            <FormControlLabel
              control={
                <Switch
                  checked={config.log_requests}
                  onChange={(e) => updateConfig({ ...config, log_requests: e.target.checked })}
                />
              }
              label={t.settings.logRequests}
            />

            <TextField
              label={t.settings.logRetentionDays}
              type="number"
              value={config.log_retention_days}
              onChange={(e) => updateConfig({ ...config, log_retention_days: parseInt(e.target.value) || 30 })}
              fullWidth
            />

            <TextField
              label={t.settings.maxRetries}
              type="number"
              value={config.max_retries}
              onChange={(e) => updateConfig({ ...config, max_retries: parseInt(e.target.value) || 3 })}
              fullWidth
            />

            <TextField
              label={t.settings.timeoutSecs}
              type="number"
              value={config.timeout_secs}
              onChange={(e) => updateConfig({ ...config, timeout_secs: parseInt(e.target.value) || 120 })}
              fullWidth
            />
          </CardContent>
        </Card>
      </Box>

      <Box>
        <Typography variant="h6" sx={{ fontWeight: 600, mb: 2 }}>{t.settings.language}</Typography>
        <Card variant="outlined" sx={{ maxWidth: 560, borderRadius: 2 }}>
          <CardContent>
            <FormControl fullWidth>
              <InputLabel>{t.settings.language}</InputLabel>
              <Select
                value={lang}
                label={t.settings.language}
                onChange={(e) => setLang(e.target.value as 'zh' | 'en')}
              >
                <MenuItem value="zh">{t.settings.zh}</MenuItem>
                <MenuItem value="en">{t.settings.en}</MenuItem>
              </Select>
            </FormControl>
          </CardContent>
        </Card>
      </Box>
    </Box>
  );
}
