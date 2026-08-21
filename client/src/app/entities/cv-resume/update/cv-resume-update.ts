import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core';
import { ReactiveFormsModule } from '@angular/forms';
import { ActivatedRoute } from '@angular/router';

import { FontAwesomeModule } from '@fortawesome/angular-fontawesome';
import { TranslatePipe } from '@ngx-translate/core';
import { Observable, finalize } from 'rxjs';

import { DataUtils, FileLoadError } from 'app/core/util/data-util.service';
import { EventManager, EventWithContent } from 'app/core/util/event-manager.service';
import { AlertError } from 'app/shared/alert/alert-error';
import { AlertErrorModel } from 'app/shared/alert/alert-error.model';
import { TranslateDirective } from 'app/shared/language';
import { ICvResume } from '../cv-resume.model';
import { CvResumeService } from '../service/cv-resume.service';

import { CvResumeFormGroup, CvResumeFormService } from './cv-resume-form.service';

@Component({
  changeDetection: ChangeDetectionStrategy.OnPush,
  selector: 'jhi-cv-resume-update',
  templateUrl: './cv-resume-update.html',
  imports: [TranslateDirective, TranslatePipe, FontAwesomeModule, AlertError, ReactiveFormsModule],
})
export class CvResumeUpdate implements OnInit {
  readonly isSaving = signal(false);
  cvResume: ICvResume | null = null;

  protected dataUtils = inject(DataUtils);
  protected eventManager = inject(EventManager);
  protected cvResumeService = inject(CvResumeService);
  protected cvResumeFormService = inject(CvResumeFormService);
  protected activatedRoute = inject(ActivatedRoute);

  // eslint-disable-next-line @typescript-eslint/member-ordering
  editForm: CvResumeFormGroup = this.cvResumeFormService.createCvResumeFormGroup();

  ngOnInit(): void {
    this.activatedRoute.data.subscribe(({ cvResume }) => {
      this.cvResume = cvResume;
      if (cvResume) {
        this.updateForm(cvResume);
      }
    });
  }

  byteSize(base64String: string): string {
    return this.dataUtils.byteSize(base64String);
  }

  openFile(base64String: string, contentType: string | null | undefined): void {
    this.dataUtils.openFile(base64String, contentType);
  }

  setFileData(event: Event, field: string, isImage: boolean): void {
    this.dataUtils.loadFileToForm(event, this.editForm, field, isImage).subscribe({
      error: (err: FileLoadError) =>
        this.eventManager.broadcast(
          new EventWithContent<AlertErrorModel>('jobSearchRustApp.error', { ...err, key: `error.file.${err.key}` }),
        ),
    });
  }

  previousState(): void {
    globalThis.history.back();
  }

  save(): void {
    this.isSaving.set(true);
    const cvResume = this.cvResumeFormService.getCvResume(this.editForm);
    if (cvResume.id === null) {
      this.subscribeToSaveResponse(this.cvResumeService.create(cvResume));
    } else {
      this.subscribeToSaveResponse(this.cvResumeService.update(cvResume));
    }
  }

  protected subscribeToSaveResponse(result: Observable<ICvResume | null>): void {
    result.pipe(finalize(() => this.onSaveFinalize())).subscribe({
      next: () => this.onSaveSuccess(),
      error: () => this.onSaveError(),
    });
  }

  protected onSaveSuccess(): void {
    this.previousState();
  }

  protected onSaveError(): void {
    // Api for inheritance.
  }

  protected onSaveFinalize(): void {
    this.isSaving.set(false);
  }

  protected updateForm(cvResume: ICvResume): void {
    this.cvResume = cvResume;
    this.cvResumeFormService.resetForm(this.editForm, cvResume);
  }
}
