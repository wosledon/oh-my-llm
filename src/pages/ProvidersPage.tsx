import { useEffect, useState } from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Table from '@mui/material/Table';
import TableHead from '@mui/material/TableHead';
import TableRow from '@mui/material/TableRow';
import TableCell from '@mui/material/TableCell';
import TableBody from '@mui/material/TableBody';
import Dialog from '@mui/material/Dialog';
import DialogTitle from '@mui/material/DialogTitle';
import DialogContent from '@mui/material/DialogContent';
import DialogActions from '@mui/material/DialogActions';
import TextField from '@mui/material/TextField';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormControl from '@mui/material/FormControl';
import CircularProgress from '@mui/material/CircularProgress';
import Card from '@mui/material/Card';
import Chip from '@mui/material/Chip';
import Alert from '@mui/material/Alert';
import Box from '@mui/material/Box';
import IconButton from '@mui/material/IconButton';
import AddIcon from '@mui/icons-material/Add';
import DeleteIcon from '@mui/icons-material/Delete';
import { useProviderStore } from '../stores/providerStore';
import { useI18n } from '../i18n';
import type { ProviderInput } from '../types';

const PROV_TYPES = ['openai', 'anthropic', 'openai_compatible'];

export default function ProvidersPage() {
  const { t } = useI18n();
  const { providers, loading, error, fetchProviders, addProvider, deleteProvider } = useProviderStore();
  const [open, setOpen] = useState(false);
  const [form, setForm] = useState<ProviderInput>({
    name: '',
    prov_type: 'openai',
    base_url: '',
    api_key: '',
    extra_headers: '',
  });

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const handleSubmit = async () => {
    await addProvider(form);
    setOpen(false);
    setForm({ name: '', prov_type: 'openai', base_url: '', api_key: '', extra_headers: '' });
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4" sx={{ fontWeight: 600 }}>{t.providers.title}</Typography>
        <Button variant="contained" startIcon={<AddIcon />} onClick={() => setOpen(true)}>
          {t.providers.addProvider}
        </Button>
      </Box>

      {loading && <CircularProgress sx={{ display: 'block', mx: 'auto', my: 4 }} />}
      {error && (
        <Alert severity="error" sx={{ mb: 2 }}>{t.common.error}: {error}</Alert>
      )}

      <Dialog open={open} onClose={() => setOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>{t.providers.addProvider}</DialogTitle>
        <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
          <TextField
            label={t.providers.name}
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
            placeholder={t.providers.placeholderName}
            fullWidth
          />
          <FormControl fullWidth>
            <InputLabel>{t.providers.type}</InputLabel>
            <Select
              value={form.prov_type}
              label={t.providers.type}
              onChange={(e) => setForm({ ...form, prov_type: e.target.value })}
            >
              {PROV_TYPES.map((pt) => (
                <MenuItem key={pt} value={pt}>{pt}</MenuItem>
              ))}
            </Select>
          </FormControl>
          <TextField
            label={t.providers.baseUrl}
            value={form.base_url}
            onChange={(e) => setForm({ ...form, base_url: e.target.value })}
            placeholder={t.providers.placeholderUrl}
            fullWidth
          />
          <TextField
            label={t.providers.apiKey}
            type="password"
            value={form.api_key}
            onChange={(e) => setForm({ ...form, api_key: e.target.value })}
            fullWidth
          />
          <TextField
            label={t.providers.extraHeaders}
            value={form.extra_headers || ''}
            onChange={(e) => setForm({ ...form, extra_headers: e.target.value })}
            placeholder={t.providers.placeholderHeaders}
            fullWidth
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setOpen(false)}>{t.providers.cancel}</Button>
          <Button variant="contained" onClick={handleSubmit}>{t.providers.save}</Button>
        </DialogActions>
      </Dialog>

      <Card variant="outlined">
        <Table>
          <TableHead>
            <TableRow>
              <TableCell>{t.providers.name}</TableCell>
              <TableCell>{t.providers.type}</TableCell>
              <TableCell>{t.providers.baseUrl}</TableCell>
              <TableCell>{t.providers.apiKey}</TableCell>
              <TableCell align="right">{t.providers.actions}</TableCell>
            </TableRow>
          </TableHead>
          <TableBody>
            {providers.map((p) => (
              <TableRow key={p.id} hover>
                <TableCell sx={{ fontWeight: 600 }}>{p.name}</TableCell>
                <TableCell>
                  <Chip size="small" variant="outlined" label={p.prov_type} />
                </TableCell>
                <TableCell sx={{ fontFamily: 'monospace', fontSize: '12px' }}>{p.base_url}</TableCell>
                <TableCell sx={{ fontFamily: 'monospace' }}>{p.api_key}</TableCell>
                <TableCell align="right">
                  <IconButton size="small" color="error" onClick={() => deleteProvider(p.id)}>
                    <DeleteIcon fontSize="small" />
                  </IconButton>
                </TableCell>
              </TableRow>
            ))}
            {providers.length === 0 && (
              <TableRow>
                <TableCell colSpan={5} align="center" sx={{ py: 6, color: 'text.secondary' }}>
                  {t.dashboard.noData}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </Card>
    </Box>
  );
}
